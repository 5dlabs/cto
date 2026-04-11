//! Skills cache: downloads agent-project tarballs from a GitHub Releases-based
//! skills repo and caches them on a PVC so they survive controller restarts.
//!
//! # Release layout
//!
//! The skills repo publishes a rolling `latest` GitHub Release with:
//!   - `hashes.txt`  — one `<sha256>  <agent>-<project>.tar.gz` line per tarball
//!   - `<agent>-default.tar.gz`   — just `_default/` skills for the agent
//!   - `<agent>-<project>.tar.gz` — `_default` merged with project overrides
//!
//! Tarball contents are always `<agent>/<skill_name>/SKILL.md` (flat, no project
//! prefix inside the archive).
//!
//! # Local cache layout
//!
//! ```text
//! <cache_root>/
//!   <agent>/                       # extracted skills (from latest tarball)
//!     <skill_name>/
//!       SKILL.md
//!       ...
//!   <agent>-<project>.hash         # persisted sha256 for change detection
//! ```
//!
//! On each reconcile the controller calls [`ensure_skills`] which:
//! 1. Fetches `hashes.txt` from the latest release.
//! 2. Compares the agent-project remote hash to the local `.hash` file.
//! 3. Downloads + extracts the tarball only if the hash changed.
//! 4. Returns the `SKILL.md` content for every requested skill.
//!
//! Any network or extraction failure is propagated as [`SkillsCacheError`] so the
//! caller can fail the CodeRun loudly — there is **no** silent fallback.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SkillsCacheError {
    #[error("failed to fetch hashes.txt from {url}: {source}")]
    FetchManifest { url: String, source: reqwest::Error },

    #[error("hashes.txt returned non-200 status {status} from {url}")]
    ManifestStatus { url: String, status: u16 },

    #[error("agent '{agent}' not found in hashes.txt manifest")]
    AgentNotInManifest { agent: String },

    #[error("failed to download {url}: {source}")]
    FetchTarball { url: String, source: reqwest::Error },

    #[error("tarball download returned non-200 status {status} from {url}")]
    TarballStatus { url: String, status: u16 },

    #[error("sha256 mismatch for agent '{agent}': expected {expected}, got {actual}")]
    HashMismatch {
        agent: String,
        expected: String,
        actual: String,
    },

    #[error("failed to extract tarball for agent '{agent}': {source}")]
    Extract {
        agent: String,
        source: std::io::Error,
    },

    #[error("SKILL.md not found in cache for skill '{skill}' (agent '{agent}')")]
    MissingSkillMd { agent: String, skill: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SkillsCacheError>;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const DEFAULT_CACHE_PATH: &str = "/data/skills-cache";

fn cache_root() -> PathBuf {
    PathBuf::from(
        std::env::var("SKILLS_CACHE_PATH").unwrap_or_else(|_| DEFAULT_CACHE_PATH.to_string()),
    )
}

/// Build the download URL for a release asset.
///
/// Given `skills_url = "https://github.com/owner/repo"` and an asset name,
/// returns `https://github.com/owner/repo/releases/download/latest/<asset>`.
fn asset_url(skills_url: &str, asset: &str) -> String {
    let base = skills_url.trim_end_matches('/');
    format!("{base}/releases/download/latest/{asset}")
}

// ---------------------------------------------------------------------------
// Manifest parsing
// ---------------------------------------------------------------------------

/// Parse `hashes.txt` content into a map of `asset_name -> sha256_hex`.
///
/// Expected format (one per line): `<hex_hash>  <asset_name>`
/// e.g. `a1b2c3...  rex.tar.gz`
fn parse_manifest(body: &str) -> HashMap<String, String> {
    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            // Format: "<hash>  <asset_name>" (two spaces, like sha256sum output)
            let (hash, name) = line.split_once("  ").or_else(|| line.split_once(' '))?;
            Some((name.trim().to_string(), hash.trim().to_string()))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

/// Ensure the agent's skills are present and up-to-date in the local cache,
/// then return `SKILL.md` content keyed by skill name.
///
/// This is the only public entry point for the module.
///
/// # Arguments
/// * `skills_url`  — base GitHub repo URL (e.g. `https://github.com/5dlabs/cto-skills`)
/// * `agent_name`  — the agent whose tarball to download (e.g. `rex`, `blaze`)
/// * `project`     — the project name (e.g. `test-sandbox`), or `None` for default
/// * `skill_names` — the skill names this CodeRun needs
///
/// # Errors
/// Returns `SkillsCacheError` on any fetch, hash, or extraction failure.
/// The caller should translate this into a CodeRun failure condition.
pub fn ensure_skills(
    skills_url: &str,
    agent_name: &str,
    project: Option<&str>,
    skill_names: &[String],
) -> Result<HashMap<String, String>> {
    let root = cache_root();
    fs::create_dir_all(&root)?;

    let project_label = project.unwrap_or("default");
    let tarball_stem = format!("{agent_name}-{project_label}");

    // 1. Fetch the manifest
    let manifest_url = asset_url(skills_url, "hashes.txt");
    debug!("Fetching skills manifest from {}", manifest_url);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to build HTTP client");

    let resp = client
        .get(&manifest_url)
        .send()
        .map_err(|e| SkillsCacheError::FetchManifest {
            url: manifest_url.clone(),
            source: e,
        })?;

    if !resp.status().is_success() {
        return Err(SkillsCacheError::ManifestStatus {
            url: manifest_url,
            status: resp.status().as_u16(),
        });
    }

    let manifest_body = resp.text().map_err(|e| SkillsCacheError::FetchManifest {
        url: manifest_url.clone(),
        source: e,
    })?;
    let manifest = parse_manifest(&manifest_body);
    debug!("Skills manifest contains {} entries", manifest.len());

    // 2. Check if the agent-project tarball needs downloading
    let tarball_key = format!("{tarball_stem}.tar.gz");
    let remote_hash =
        manifest
            .get(&tarball_key)
            .ok_or_else(|| SkillsCacheError::AgentNotInManifest {
                agent: tarball_stem.clone(),
            })?;

    // All tarballs extract into {agent}/ so we key the cache on agent_name,
    // but track hashes per agent-project pair.
    let agent_dir = root.join(agent_name);
    let hash_file = root.join(format!("{tarball_stem}.hash"));

    let needs_download = if hash_file.exists() && agent_dir.exists() {
        let local_hash = fs::read_to_string(&hash_file).unwrap_or_default();
        let local_hash = local_hash.trim();
        if local_hash == remote_hash {
            debug!(
                "Skills for '{tarball_stem}' are up-to-date (hash {})",
                &remote_hash[..remote_hash.len().min(8)]
            );
            false
        } else {
            info!(
                "Skills for '{tarball_stem}' hash changed: {} -> {}",
                &local_hash[..local_hash.len().min(8)],
                &remote_hash[..remote_hash.len().min(8)]
            );
            true
        }
    } else {
        info!("Skills for '{tarball_stem}' not in cache, downloading",);
        true
    };

    if needs_download {
        download_and_extract(
            &client,
            skills_url,
            &tarball_stem,
            agent_name,
            remote_hash,
            &root,
        )?;
    }

    // 3. Read SKILL.md content for each requested skill
    let mut result = HashMap::with_capacity(skill_names.len());

    for name in skill_names {
        let skill_md_path = agent_dir.join(name).join("SKILL.md");
        match fs::read_to_string(&skill_md_path) {
            Ok(content) => {
                debug!("Loaded skill '{}' for agent '{}'", name, agent_name);
                result.insert(name.clone(), content);
            }
            Err(_) => {
                // Skill might not be in this agent's tarball (e.g. optional skill
                // not included in _default). Log but don't fail — the caller can
                // decide whether an empty skill is acceptable.
                warn!(
                    "Skill '{}' not found in agent '{}' cache at {}",
                    name,
                    agent_name,
                    skill_md_path.display()
                );
            }
        }
    }

    Ok(result)
}

/// Persona file names that are read from the `_persona/` subdirectory of
/// the agent's cached tarball.
const PERSONA_FILES: &[&str] = &[
    "AGENTS.md",
    "SOUL.md",
    "USER.md",
    "IDENTITY.md",
    "TOOLS.md",
    "HEARTBEAT.md",
    "BOOT.md",
];

/// Read persona/personality files from the agent's cached tarball directory.
///
/// Must be called **after** [`ensure_skills`] so the tarball is already extracted.
/// Returns a map of `filename -> content` for each persona file found
/// (e.g. `"AGENTS.md" -> "# Rex — Operating Instructions\n..."`).
///
/// Missing files are silently skipped — not every agent has every persona file.
pub fn get_persona_files(agent_name: &str) -> HashMap<String, String> {
    let persona_dir = cache_root().join(agent_name).join("_persona");
    let mut result = HashMap::new();

    if !persona_dir.exists() {
        debug!("No _persona/ directory for agent '{agent_name}'");
        return result;
    }

    for filename in PERSONA_FILES {
        let path = persona_dir.join(filename);
        if let Ok(content) = fs::read_to_string(&path) {
            debug!("Loaded persona file '{filename}' for agent '{agent_name}'");
            result.insert((*filename).to_string(), content);
        }
    }

    info!(
        "Loaded {} persona files for agent '{agent_name}'",
        result.len()
    );
    result
}
///
/// `tarball_stem` is the release asset stem (e.g. `rex-default`, `rex-test-sandbox`).
/// `agent_name` is the agent directory inside the tarball (e.g. `rex`).
fn download_and_extract(
    client: &reqwest::blocking::Client,
    skills_url: &str,
    tarball_stem: &str,
    agent_name: &str,
    expected_hash: &str,
    cache_root: &Path,
) -> Result<()> {
    let tarball_url = asset_url(skills_url, &format!("{tarball_stem}.tar.gz"));
    debug!("Downloading skills tarball from {}", tarball_url);

    let resp = client
        .get(&tarball_url)
        .send()
        .map_err(|e| SkillsCacheError::FetchTarball {
            url: tarball_url.clone(),
            source: e,
        })?;

    if !resp.status().is_success() {
        return Err(SkillsCacheError::TarballStatus {
            url: tarball_url,
            status: resp.status().as_u16(),
        });
    }

    let bytes = resp.bytes().map_err(|e| SkillsCacheError::FetchTarball {
        url: tarball_url.clone(),
        source: e,
    })?;

    // Verify hash before extracting
    let actual_hash = hex::encode(Sha256::digest(&bytes));
    if actual_hash != expected_hash {
        return Err(SkillsCacheError::HashMismatch {
            agent: tarball_stem.to_string(),
            expected: expected_hash.to_string(),
            actual: actual_hash,
        });
    }

    // Remove old agent directory if present, then extract
    let agent_dir = cache_root.join(agent_name);
    if agent_dir.exists() {
        fs::remove_dir_all(&agent_dir).map_err(|e| SkillsCacheError::Extract {
            agent: tarball_stem.to_string(),
            source: e,
        })?;
    }

    let decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(cache_root)
        .map_err(|e| SkillsCacheError::Extract {
            agent: tarball_stem.to_string(),
            source: e,
        })?;

    // Persist the hash keyed by tarball_stem
    let hash_file = cache_root.join(format!("{tarball_stem}.hash"));
    fs::write(&hash_file, expected_hash)?;

    // Count extracted skills
    let skill_count = fs::read_dir(&agent_dir)
        .map(|rd| {
            rd.filter(|e| e.as_ref().is_ok_and(|e| e.path().is_dir()))
                .count()
        })
        .unwrap_or(0);

    info!(
        "Extracted {} skills for '{}' (hash {})",
        skill_count,
        tarball_stem,
        &expected_hash[..expected_hash.len().min(8)]
    );
    Ok(())
}

// Inline hex encoding to avoid adding the `hex` crate just for this.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .fold(String::with_capacity(64), |mut s, b| {
                use std::fmt::Write;
                let _ = write!(s, "{b:02x}");
                s
            })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let body = r"
abc123def456  rex-default.tar.gz
# comment line
789aaa000bbb  rex-test-sandbox.tar.gz
";
        let m = parse_manifest(body);
        assert_eq!(m.len(), 2);
        assert_eq!(m.get("rex-default.tar.gz").unwrap(), "abc123def456");
        assert_eq!(m.get("rex-test-sandbox.tar.gz").unwrap(), "789aaa000bbb");
    }

    #[test]
    fn test_parse_manifest_single_space() {
        let body = "deadbeef blaze-default.tar.gz\n";
        let m = parse_manifest(body);
        assert_eq!(m.get("blaze-default.tar.gz").unwrap(), "deadbeef");
    }

    #[test]
    fn test_asset_url() {
        let url = asset_url("https://github.com/org/repo", "rex-default.tar.gz");
        assert_eq!(
            url,
            "https://github.com/org/repo/releases/download/latest/rex-default.tar.gz"
        );
    }

    #[test]
    fn test_asset_url_trailing_slash() {
        let url = asset_url("https://github.com/org/repo/", "hashes.txt");
        assert_eq!(
            url,
            "https://github.com/org/repo/releases/download/latest/hashes.txt"
        );
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode([0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn test_agent_tarball_extraction_roundtrip() {
        // Create a tarball with agent/{skill}/SKILL.md structure
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();

        let mut tar_bytes = Vec::new();
        {
            let encoder =
                flate2::write::GzEncoder::new(&mut tar_bytes, flate2::Compression::fast());
            let mut builder = tar::Builder::new(encoder);

            // Add two skills under the agent directory
            let content1 = b"# Rust Patterns\nRust skill content.";
            let mut header1 = tar::Header::new_gnu();
            header1.set_size(content1.len() as u64);
            header1.set_mode(0o644);
            header1.set_cksum();
            builder
                .append_data(&mut header1, "rex/rust-patterns/SKILL.md", &content1[..])
                .unwrap();

            let content2 = b"# Test Sandbox\nProject-specific skill.";
            let mut header2 = tar::Header::new_gnu();
            header2.set_size(content2.len() as u64);
            header2.set_mode(0o644);
            header2.set_cksum();
            builder
                .append_data(
                    &mut header2,
                    "rex/test-sandbox-guidelines/SKILL.md",
                    &content2[..],
                )
                .unwrap();

            builder.into_inner().unwrap().finish().unwrap();
        }

        // Compute hash
        let hash = hex::encode(Sha256::digest(&tar_bytes));

        // Extract
        let decoder = flate2::read::GzDecoder::new(&tar_bytes[..]);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(cache).unwrap();

        // Write hash file
        fs::write(cache.join("rex.hash"), &hash).unwrap();

        // Verify both skills extracted
        let skill1 = fs::read_to_string(cache.join("rex/rust-patterns/SKILL.md")).unwrap();
        assert_eq!(skill1, "# Rust Patterns\nRust skill content.");

        let skill2 =
            fs::read_to_string(cache.join("rex/test-sandbox-guidelines/SKILL.md")).unwrap();
        assert_eq!(skill2, "# Test Sandbox\nProject-specific skill.");

        let stored_hash = fs::read_to_string(cache.join("rex.hash")).unwrap();
        assert_eq!(stored_hash, hash);
    }
}
