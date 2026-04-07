use std::fmt::Write;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use base64::Engine;
use serde::Deserialize;

use crate::{CodeSearchResult, MergeRequest, RepoInfo, ScmClient, ScmClientConfig, ScmProvider};

pub struct GitHubClient {
    http: reqwest::Client,
    api_base: String,
    host: String,
    token: Option<String>,
}

impl GitHubClient {
    #[must_use]
    pub fn new(config: &ScmClientConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_base: config.api_base.clone(),
            host: config.host.clone(),
            token: config.token.clone(),
        }
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());
        headers.insert("User-Agent", "cto-scm/1.0".parse().unwrap());
        if let Some(ref token) = self.token {
            headers.insert("Authorization", format!("Bearer {token}").parse().unwrap());
        }
        headers
    }
}

#[derive(Deserialize)]
struct GhPr {
    number: u64,
    title: String,
    state: String,
    html_url: String,
    head: GhRef,
    base: GhRef,
    user: GhUser,
}

#[derive(Deserialize)]
struct GhRef {
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Deserialize)]
struct GhUser {
    login: String,
}

#[derive(Deserialize)]
struct GhRepo {
    full_name: String,
    html_url: String,
    clone_url: String,
    default_branch: String,
}

#[derive(Deserialize)]
struct GhContent {
    content: Option<String>,
    encoding: Option<String>,
}

#[derive(Deserialize)]
struct GhSearchResult {
    items: Vec<GhSearchItem>,
}

#[derive(Deserialize)]
struct GhSearchItem {
    path: String,
    html_url: String,
    repository: GhSearchRepo,
    #[serde(default)]
    text_matches: Vec<GhTextMatch>,
}

#[derive(Deserialize)]
struct GhSearchRepo {
    full_name: String,
}

#[derive(Deserialize)]
struct GhTextMatch {
    fragment: Option<String>,
}

#[derive(Deserialize)]
struct GhIssue {
    html_url: String,
}

impl From<GhPr> for MergeRequest {
    fn from(pr: GhPr) -> Self {
        Self {
            number: pr.number,
            title: pr.title,
            state: pr.state,
            source_branch: pr.head.ref_name,
            target_branch: pr.base.ref_name,
            url: pr.html_url,
            author: pr.user.login,
        }
    }
}

#[async_trait]
impl ScmClient for GitHubClient {
    async fn list_open_mrs(
        &self,
        owner: &str,
        repo: &str,
        head_branch: Option<&str>,
    ) -> Result<Vec<MergeRequest>> {
        let mut url = format!("{}/repos/{owner}/{repo}/pulls?state=open", self.api_base);
        if let Some(branch) = head_branch {
            let _ = write!(url, "&head={owner}:{branch}");
        }
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("GitHub list PRs request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitHub list PRs returned {status}: {body}");
        }
        let prs: Vec<GhPr> = resp.json().await?;
        Ok(prs.into_iter().map(MergeRequest::from).collect())
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
        let url = format!("{}/repos/{owner}/{repo}/pulls", self.api_base);
        let payload = serde_json::json!({
            "title": title,
            "body": body,
            "head": head,
            "base": base,
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitHub create PR request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            bail!("GitHub create PR returned {status}: {body_text}");
        }
        let pr: GhPr = resp.json().await?;
        Ok(pr.into())
    }

    async fn get_file_contents(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        ref_: &str,
    ) -> Result<Vec<u8>> {
        let url = format!(
            "{}/repos/{owner}/{repo}/contents/{path}?ref={ref_}",
            self.api_base
        );
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("GitHub get file request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitHub get file returned {status}: {body}");
        }
        let content: GhContent = resp.json().await?;
        match (content.content, content.encoding.as_deref()) {
            (Some(b64), Some("base64")) => {
                let cleaned = b64.replace(['\n', '\r'], "");
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(cleaned)
                    .context("failed to decode base64 file content")?;
                Ok(decoded)
            }
            (Some(raw), _) => Ok(raw.into_bytes()),
            _ => bail!("no content in GitHub file response"),
        }
    }

    async fn create_repo(&self, org: &str, name: &str, private: bool) -> Result<RepoInfo> {
        let url = format!("{}/orgs/{org}/repos", self.api_base);
        let payload = serde_json::json!({
            "name": name,
            "private": private,
            "auto_init": true,
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitHub create repo request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitHub create repo returned {status}: {body}");
        }
        let repo: GhRepo = resp.json().await?;
        Ok(RepoInfo {
            full_name: repo.full_name,
            url: repo.html_url,
            clone_url: repo.clone_url,
            default_branch: repo.default_branch,
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
        let api_url = format!("{}/repos/{owner}/{repo}/hooks", self.api_base);
        let mut config = serde_json::json!({
            "url": url,
            "content_type": "json",
            "insecure_ssl": "0",
        });
        if let Some(s) = secret {
            config["secret"] = serde_json::Value::String(s.to_string());
        }
        let payload = serde_json::json!({
            "name": "web",
            "config": config,
            "events": events,
            "active": true,
        });
        let resp = self
            .http
            .post(&api_url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitHub create webhook request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("GitHub create webhook returned {status}: {body}");
        }
        Ok(())
    }

    async fn search_code(&self, query: &str) -> Result<Vec<CodeSearchResult>> {
        let url = format!(
            "{}/search/code?q={}",
            self.api_base,
            urlencoding::encode(query)
        );
        let mut headers = self.auth_headers();
        headers.insert(
            "Accept",
            "application/vnd.github.text-match+json".parse().unwrap(),
        );
        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("GitHub search code request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("GitHub search code returned {status}: {body}");
        }
        let result: GhSearchResult = resp.json().await?;
        Ok(result
            .items
            .into_iter()
            .map(|item| CodeSearchResult {
                path: item.path,
                repository: item.repository.full_name,
                url: item.html_url,
                fragment: item.text_matches.first().and_then(|m| m.fragment.clone()),
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
        let url = format!("{}/repos/{owner}/{repo}/issues", self.api_base);
        let payload = serde_json::json!({
            "title": title,
            "body": body,
            "labels": labels,
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&payload)
            .send()
            .await
            .context("GitHub create issue request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            bail!("GitHub create issue returned {status}: {body_text}");
        }
        let issue: GhIssue = resp.json().await?;
        Ok(issue.html_url)
    }

    fn mr_url(&self, owner: &str, repo: &str, number: u64) -> String {
        format!("https://{}/{owner}/{repo}/pull/{number}", self.host)
    }

    fn repo_url(&self, owner: &str, repo: &str) -> String {
        format!("https://{}/{owner}/{repo}", self.host)
    }

    fn clone_url(&self, owner: &str, repo: &str, token: Option<&str>) -> String {
        match token {
            Some(t) => format!(
                "https://x-access-token:{t}@{}/{owner}/{repo}.git",
                self.host
            ),
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
            .context("URL does not match configured GitHub host")?;
        let parts: Vec<&str> = path.splitn(3, '/').collect();
        if parts.len() < 2 {
            bail!("cannot parse owner/repo from URL: {url}");
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    fn provider(&self) -> ScmProvider {
        ScmProvider::GitHub
    }
}
