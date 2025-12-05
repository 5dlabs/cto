//! Browser-based authentication using chromiumoxide.

use anyhow::Result;
use std::path::Path;

use super::Session;

/// Browser-based authentication for Twitter/X.
pub struct BrowserAuth {
    /// Whether to run in headless mode.
    headless: bool,
}

impl BrowserAuth {
    /// Create a new browser auth instance.
    #[must_use]
    pub fn new(headless: bool) -> Self {
        Self { headless }
    }

    /// Perform interactive login and extract session cookies.
    ///
    /// This opens a browser window for the user to log in manually.
    pub async fn login(&self) -> Result<Session> {
        use chromiumoxide::browser::{Browser, BrowserConfig};
        use futures::StreamExt;

        tracing::info!(headless = self.headless, "Launching browser for login");

        let config = if self.headless {
            BrowserConfig::builder()
                .arg("--no-sandbox") // Required for containerized environments
                .arg("--disable-dev-shm-usage") // Avoid /dev/shm size issues in containers
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build browser config: {e}"))?
        } else {
            BrowserConfig::builder()
                .with_head()
                .arg("--no-sandbox")
                .arg("--disable-dev-shm-usage")
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build browser config: {e}"))?
        };

        let (mut browser, mut handler) = Browser::launch(config).await?;

        // Spawn handler task
        let handle = tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        // Navigate to Twitter login
        let page = browser.new_page("https://x.com/login").await?;

        if !self.headless {
            println!("\n🔐 Please log in to Twitter/X in the browser window.");
            println!("   Press Enter when you're done...\n");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
        }

        // Wait a bit for cookies to settle
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Extract cookies via CDP (document.cookie can't access HttpOnly cookies like auth_token)
        let cookies = page.get_cookies().await?;

        // Parse cookies
        let mut auth_token = None;
        let mut ct0 = None;

        for cookie in cookies {
            match cookie.name.as_str() {
                "auth_token" => auth_token = Some(cookie.value.clone()),
                "ct0" => ct0 = Some(cookie.value.clone()),
                _ => {}
            }
        }

        browser.close().await?;
        handle.await?;

        let auth_token = auth_token.ok_or_else(|| {
            anyhow::anyhow!("Failed to extract auth_token cookie - login may have failed")
        })?;

        let session = if let Some(ct0) = ct0 {
            Session::with_ct0(auth_token, ct0)
        } else {
            Session::new(auth_token)
        };

        tracing::info!("Successfully extracted session cookies");
        Ok(session)
    }

    /// Validate an existing session by making a test request.
    pub async fn validate_session(&self, session: &Session) -> Result<bool> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()?;

        let response = client
            .get("https://x.com/i/api/1.1/account/settings.json")
            .header("Cookie", session.cookie_string())
            .header("x-csrf-token", session.ct0.as_deref().unwrap_or(""))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Load session from file, validating if it exists.
    pub async fn load_or_login(&self, session_path: &Path) -> Result<Session> {
        if session_path.exists() {
            tracing::info!(path = %session_path.display(), "Loading existing session");
            let mut session = Session::load(session_path)?;

            if self.validate_session(&session).await? {
                tracing::info!("Session is valid");
                session.mark_validated();
                session.save(session_path)?;
                return Ok(session);
            }

            tracing::warn!("Session expired, need to re-authenticate");
        }

        let session = self.login().await?;
        session.save(session_path)?;
        Ok(session)
    }
}
