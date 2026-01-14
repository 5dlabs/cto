use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::sync::LazyLock;
use tracing::{debug, warn};

const TOOLS_CATALOG_PATH: &str = "/tools-catalog/tool-catalog.json";

static TOOL_CATALOG: LazyLock<ToolCatalog> = LazyLock::new(ToolCatalog::load);

/// Resolve a requested remote tool name to the canonical identifier advertised by Tools.
/// Returns `None` when the name is invalid (not present in the catalog) and cannot be
/// normalized via the legacy heuristics.
///
/// NOTE: This function falls back to returning the unchanged name when:
/// - The catalog is not loaded (graceful degradation)
/// - The tool is not found in the catalog (backward compatibility)
///
/// Use [`try_resolve_tool_strict`] if you need to distinguish between
/// "tool found" vs "tool not found" for validation purposes.
pub fn resolve_tool_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    match TOOL_CATALOG.resolve(trimmed) {
        Some(canonical) => Some(canonical),
        None if !TOOL_CATALOG.is_loaded() => Some(trimmed.to_string()),
        None => {
            let variants = vec![trimmed.replace('-', "_"), trimmed.replace('_', "-")];

            for variant in variants {
                if variant != trimmed {
                    if let Some(found) = TOOL_CATALOG.resolve(&variant) {
                        return Some(found);
                    }
                }
            }

            warn!(
                tool = trimmed,
                "remote tool not present in Tools catalog; leaving name unchanged"
            );
            Some(trimmed.to_string())
        }
    }
}

/// Result of strict tool resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolResolutionResult {
    /// Tool was found in the catalog - canonical name returned
    Resolved(String),
    /// Tool was NOT found in the catalog (catalog is loaded)
    NotFound,
    /// Catalog is not loaded - cannot determine if tool exists
    CatalogUnavailable,
}

/// Strictly resolve a tool name, distinguishing between "found", "not found", and "unknown".
///
/// Unlike [`resolve_tool_name`], this function does NOT fall back to returning the
/// unchanged name. Instead, it returns a [`ToolResolutionResult`] that clearly indicates:
/// - `Resolved(canonical)` - tool was found in catalog
/// - `NotFound` - catalog is loaded but tool doesn't exist
/// - `CatalogUnavailable` - catalog couldn't be loaded, resolution impossible
///
/// This is useful for tool inventory validation where you need to detect misconfigurations.
pub fn try_resolve_tool_strict(name: &str) -> ToolResolutionResult {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return ToolResolutionResult::NotFound;
    }

    if !TOOL_CATALOG.is_loaded() {
        return ToolResolutionResult::CatalogUnavailable;
    }

    // Try direct resolution
    if let Some(canonical) = TOOL_CATALOG.resolve(trimmed) {
        return ToolResolutionResult::Resolved(canonical);
    }

    // Try variant resolution (underscore/dash normalization)
    let variants = vec![trimmed.replace('-', "_"), trimmed.replace('_', "-")];
    for variant in variants {
        if variant != trimmed {
            if let Some(found) = TOOL_CATALOG.resolve(&variant) {
                return ToolResolutionResult::Resolved(found);
            }
        }
    }

    // Tool not found in catalog
    ToolResolutionResult::NotFound
}

struct ToolCatalog {
    lookup: HashMap<String, String>,
    loaded: bool,
}

impl ToolCatalog {
    fn load() -> Self {
        match fs::read_to_string(TOOLS_CATALOG_PATH) {
            Ok(raw) => match serde_json::from_str::<Value>(&raw) {
                Ok(json) => {
                    let mut lookup = HashMap::new();

                    if let Some(remote) = json.get("remote").and_then(|v| v.as_object()) {
                        for (server_name, server_value) in remote {
                            if let Some(tools) =
                                server_value.get("tools").and_then(|v| v.as_array())
                            {
                                let server_variants = server_variants(server_name);

                                for tool in tools {
                                    if let Some(tool_name) =
                                        tool.get("name").and_then(|v| v.as_str())
                                    {
                                        let tool_variants = tool_variants(tool_name);
                                        let canonical = canonical_name(server_name, tool_name);

                                        for variant in
                                            variant_matrix(&server_variants, &tool_variants)
                                        {
                                            insert_variant(&mut lookup, &variant, &canonical);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    debug!(loaded = lookup.len(), "Loaded Tools tool catalog entries");

                    Self {
                        lookup,
                        loaded: true,
                    }
                }
                Err(err) => {
                    warn!(
                        error = %err,
                        path = TOOLS_CATALOG_PATH,
                        "Failed to parse Tools catalog JSON"
                    );
                    Self::empty_with_warning()
                }
            },
            Err(err) => {
                warn!(
                    error = %err,
                    path = TOOLS_CATALOG_PATH,
                    "Tools catalog not mounted; remote tool validation disabled"
                );
                Self::empty_with_warning()
            }
        }
    }

    fn resolve(&self, name: &str) -> Option<String> {
        if !self.loaded || self.lookup.is_empty() {
            return None;
        }

        self.lookup
            .get(name)
            .cloned()
            .or_else(|| self.lookup.get(&name.to_lowercase()).cloned())
    }

    fn empty_with_warning() -> Self {
        warn!("Tools catalog unavailable; remote tool names must already match canonical values");
        Self {
            lookup: HashMap::new(),
            loaded: false,
        }
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }
}

fn insert_variant(map: &mut HashMap<String, String>, variant: &str, canonical: &str) {
    if variant.is_empty() {
        return;
    }

    map.entry(variant.to_string())
        .or_insert_with(|| canonical.to_string());

    let lowercase = variant.to_lowercase();
    map.entry(lowercase)
        .or_insert_with(|| canonical.to_string());
}

fn server_variants(server: &str) -> BTreeSet<String> {
    let mut variants = BTreeSet::new();
    variants.insert(server.to_string());
    variants.insert(server.replace('-', "_"));
    variants.insert(server.replace('_', "-"));
    variants
}

fn tool_variants(tool: &str) -> BTreeSet<String> {
    let mut variants = BTreeSet::new();
    variants.insert(tool.to_string());
    variants.insert(tool.replace('-', "_"));
    variants.insert(tool.replace('_', "-"));
    variants
}

fn variant_matrix(
    server_variants: &BTreeSet<String>,
    tool_variants: &BTreeSet<String>,
) -> Vec<String> {
    let mut variants = Vec::new();
    for server in server_variants {
        for tool in tool_variants {
            variants.push(format!("{server}_{tool}"));
        }
    }
    variants
}

fn canonical_name(server: &str, tool: &str) -> String {
    format!("{}_{}", server.replace('-', "_"), tool)
}
