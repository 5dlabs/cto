use std::fmt::Write;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use base64::Engine;
use serde::Deserialize;

use crate::{CodeSearchResult, MergeRequest, RepoInfo, ScmClient, ScmClientConfig, ScmProvider};

pub struct GitLabClient {
    http: reqwest::Client,
    api_base: String,
    host: String,
    token: Option<String>,
}

impl GitLabClient {
    #[must_use]
    pub fn new(config: &ScmClientConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_base: config.api_base.trim_end_matches('/').to_string(),
            host: config.host.clone(),
            token: config.token.clone(),
        }
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("User-Agent", "cto-scm/1.0".parse().unwrap());
        if let Some(ref token) = self.token {
            headers.insert("PRIVATE-TOKEN", token.parse().unwrap());
        }
        headers
    }

    /// GitLab API uses URL-encoded `group/project` as the project identifier.
    fn project_id(owner: &str, repo: &str) -> String {
        urlencoding::encode(&format!("{owner}/{repo}")).into_owned()
    }
}

#[derive(Deserialize)]
struct GlMr {
    iid: u64,
    title: String,
    state: String,
    source_branch: String,
    target_branch: String,
    web_url: String,
    author: GlUser,
}

#[derive(Deserialize)]
struct GlUser {
    username: String,
}

#[derive(Deserialize)]
struct GlProject {
    path_with_namespace: String,
    web_url: String,
    http_url_to_repo: String,
    default_branch: Option<String>,
}

#[derive(Deserialize)]
struct GlFile {
    content: String,
    encoding: String,
}

#[derive(Deserialize)]
struct GlSearchBlob {
    path: String,
    #[serde(rename = "ref")]
    ref_name: String,
    data: Option<String>,
    project_id: u64,
}

#[derive(Deserialize)]
struct GlIssue {
    web_url: String,
}

impl From<GlMr> for MergeRequest {
    fn from(mr: GlMr) -> Self {
        Self {
            number: mr.iid,
            title: mr.title,
            state: mr.state,
            source_branch: mr.source_branch,
            target_branch: mr.target_branch,
            url: mr.web_url,
            author: mr.author.username,
        }
    }
}

#[async_trait]
impl ScmClient for GitLabClient {
    async fn list_open_mrs(
        &self,
        owner: &str,
        repo: &str,
        head_branch: Option<&str>,
    ) -> Result<Vec<MergeRequest>> {
        let pid = Self::project_id(owner, repo);
        let mut url = format!(
            "{}/projects/{pid}/merge_requests?state=opened",
            self.api_base
        );
        if let Some(branch) = head_branch {
            let _ = write!(url, "&source_branch={branch}");
        }
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("GitLab list MRs request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitLab list MRs returned {status}: {body}");
        }
        let mrs: Vec<GlMr> = resp.json().await?;
        Ok(mrs.into_iter().map(MergeRequest::from).collect())
    }

    async fn create_mr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<MergeRequest> {
        let pid = Self::project_id(owner, repo);
        let url = format!("{}/projects/{pid}/merge_requests", self.api_base);
        let payload = serde_json::json!({
            "title": title,
            "description": body,
            "source_branch": head,
            "target_branch": base,
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitLab create MR request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            bail!("GitLab create MR returned {status}: {body_text}");
        }
        let mr: GlMr = resp.json().await?;
        Ok(mr.into())
    }

    async fn get_file_contents(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        ref_: &str,
    ) -> Result<Vec<u8>> {
        let pid = Self::project_id(owner, repo);
        let encoded_path = urlencoding::encode(path);
        let url = format!(
            "{}/projects/{pid}/repository/files/{encoded_path}?ref={ref_}",
            self.api_base
        );
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("GitLab get file request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitLab get file returned {status}: {body}");
        }
        let file: GlFile = resp.json().await?;
        if file.encoding == "base64" {
            let cleaned = file.content.replace(['\n', '\r'], "");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(cleaned)
                .context("failed to decode base64 file content")?;
            Ok(decoded)
        } else {
            Ok(file.content.into_bytes())
        }
    }

    async fn create_repo(&self, org: &str, name: &str, private: bool) -> Result<RepoInfo> {
        let url = format!("{}/projects", self.api_base);
        let visibility = if private { "private" } else { "public" };
        let payload = serde_json::json!({
            "name": name,
            "namespace_path": org,
            "visibility": visibility,
            "initialize_with_readme": true,
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitLab create project request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitLab create project returned {status}: {body}");
        }
        let project: GlProject = resp.json().await?;
        Ok(RepoInfo {
            full_name: project.path_with_namespace,
            url: project.web_url,
            clone_url: project.http_url_to_repo,
            default_branch: project.default_branch.unwrap_or_else(|| "main".to_string()),
        })
    }

    async fn create_webhook(
        &self,
        owner: &str,
        repo: &str,
        url: &str,
        events: &[&str],
        secret: Option<&str>,
    ) -> Result<()> {
        let pid = Self::project_id(owner, repo);
        let api_url = format!("{}/projects/{pid}/hooks", self.api_base);

        let mut payload = serde_json::json!({
            "url": url,
            "push_events": false,
            "enable_ssl_verification": true,
        });

        for event in events {
            match *event {
                "merge_request" | "pull_request" => {
                    payload["merge_requests_events"] = serde_json::Value::Bool(true);
                }
                "pipeline" | "check_run" => {
                    payload["pipeline_events"] = serde_json::Value::Bool(true);
                }
                "note" | "issue_comment" | "pull_request_review_comment" => {
                    payload["note_events"] = serde_json::Value::Bool(true);
                }
                "push" => {
                    payload["push_events"] = serde_json::Value::Bool(true);
                }
                _ => {
                    tracing::warn!("unrecognised webhook event for GitLab: {event}");
                }
            }
        }

        if let Some(s) = secret {
            payload["token"] = serde_json::Value::String(s.to_string());
        }

        let resp = self
            .http
            .post(&api_url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitLab create webhook request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("GitLab create webhook returned {status}: {body}");
        }
        Ok(())
    }

    async fn search_code(&self, query: &str) -> Result<Vec<CodeSearchResult>> {
        let encoded = urlencoding::encode(query);
        let url = format!("{}/search?scope=blobs&search={encoded}", self.api_base);
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("GitLab search code request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitLab search code returned {status}: {body}");
        }
        let blobs: Vec<GlSearchBlob> = resp.json().await?;
        Ok(blobs
            .into_iter()
            .map(|blob| CodeSearchResult {
                path: blob.path.clone(),
                repository: blob.project_id.to_string(),
                url: format!(
                    "https://{}/projects/{}/-/blob/{}/{}",
                    self.host, blob.project_id, blob.ref_name, blob.path
                ),
                fragment: blob.data,
            })
            .collect())
    }

    async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<String> {
        let pid = Self::project_id(owner, repo);
        let url = format!("{}/projects/{pid}/issues", self.api_base);
        let payload = serde_json::json!({
            "title": title,
            "description": body,
            "labels": labels.join(","),
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitLab create issue request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            bail!("GitLab create issue returned {status}: {body_text}");
        }
        let issue: GlIssue = resp.json().await?;
        Ok(issue.web_url)
    }

    fn mr_url(&self, owner: &str, repo: &str, number: u64) -> String {
        format!(
            "https://{}/{owner}/{repo}/-/merge_requests/{number}",
            self.host
        )
    }

    fn repo_url(&self, owner: &str, repo: &str) -> String {
        format!("https://{}/{owner}/{repo}", self.host)
    }

    fn clone_url(&self, owner: &str, repo: &str, token: Option<&str>) -> String {
        match token {
            Some(t) => {
                format!("https://oauth2:{t}@{}/{owner}/{repo}.git", self.host)
            }
            None => format!("https://{}/{owner}/{repo}.git", self.host),
        }
    }

    fn parse_repo_from_url(&self, url: &str) -> Result<(String, String)> {
        let cleaned = url.trim_end_matches('/').trim_end_matches(".git").replace(
            &format!("git@{}:", self.host),
            &format!("https://{}/", self.host),
        );
        let prefix = format!("https://{}/", self.host);
        let path = cleaned
            .strip_prefix(&prefix)
            .context("URL does not match configured GitLab host")?;
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        if parts.len() < 2 {
            bail!("cannot parse group/project from URL: {url}");
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    fn provider(&self) -> ScmProvider {
        ScmProvider::GitLab
    }
}
