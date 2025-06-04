use axum::{Router, routing::get, Json};
use serde::Serialize;
use std::process::Command;
use std::sync::Arc;
use anyhow::{anyhow, Result};

#[derive(Serialize)]
pub struct GitInfo {
    commit_hash: String,
    repo_url: String,
}

pub struct GitService {
    repo_url: String,
}

impl GitService {
    pub fn new(repo_url: String) -> Self {
        Self { repo_url }
    }

    fn get_remote_url() -> Result<String> {
        let output = Command::new("git")
            .args(["config", "--get", "remote.origin.url"])
            .output()?;

        let url = String::from_utf8(output.stdout)?;
        let url = url.trim();

        // Convert SSH URLs to HTTPS URLs
        if url.starts_with("git@github.com:") {
            Ok(url
                .replace("git@github.com:", "https://github.com/")
                .replace(".git", ""))
        } else if url.starts_with("https://") {
            Ok(url.replace(".git", "").to_string())
        } else {
            Err(anyhow!("Invalid remote URL: {}", url))
        }
    }

    async fn get_info(&self) -> GitInfo {
        let commit_hash = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|hash| hash.trim().to_string())
            .unwrap_or_default();

        if self.repo_url.len() == 0 {
            GitInfo {
                commit_hash,
                repo_url: GitService::get_remote_url().unwrap_or_default()
            }
        } else {
            GitInfo {
                commit_hash,
                repo_url: self.repo_url.clone()
            }
        }
    }
}

async fn get_git_info(
    axum::extract::State(service): axum::extract::State<Arc<GitService>>,
) -> Json<GitInfo> {
    Json(service.get_info().await)
}

pub fn git_info(repo_url: String) -> Router {
    let service = Arc::new(GitService::new(repo_url));
    
    Router::new()
        .route("/git_info", get(get_git_info))
        .with_state(service)
}