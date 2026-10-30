//! "A new version is available" check against GitHub releases. Notification only: the user
//! downloads the new installer from the website themselves. No auto-install.

use std::time::Duration;

use serde::{Deserialize, Serialize};

const LATEST_RELEASE_API: &str = "https://api.github.com/repos/Azmekk/VideoCrop/releases/latest";
/// Where "Download" leads, and the only URL `open_release_page` will hand to the shell.
pub const DOWNLOAD_PAGE: &str = "https://videocrop.vexbyte.com/";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Without the leading "v", e.g. "2.1.0".
    pub version: String,
    pub url: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// `v2.1.0` / `2.1` -> (2, 1, 0). Pre-release tags (`-beta`) return None so they never prompt.
pub fn parse_version(tag: &str) -> Option<(u64, u64, u64)> {
    let t = tag.trim().trim_start_matches(['v', 'V']);
    if t.contains('-') {
        return None;
    }
    let mut parts = t.split('.').map(|p| p.parse::<u64>().ok());
    let major = parts.next()??;
    let minor = parts.next().unwrap_or(Some(0))?;
    let patch = parts.next().unwrap_or(Some(0))?;
    Some((major, minor, patch))
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

/// Blocking; call from `spawn_blocking`. `Ok(None)` = up to date, no releases, or offline-ish.
pub fn check(current_version: &str) -> Result<Option<UpdateInfo>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .user_agent(concat!("VideoCrop/", env!("CARGO_PKG_VERSION")))
        .http_status_as_error(false)
        .build()
        .into();
    let mut resp = agent
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("Update check failed: {e}"))?;
    if resp.status() == 404 {
        return Ok(None); // no releases published yet
    }
    if !resp.status().is_success() {
        return Err(format!("Update check failed: HTTP {}", resp.status()));
    }
    let body = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Update check failed: {e}"))?;
    let release: Release = serde_json::from_str(&body).map_err(|e| format!("Update check failed: {e}"))?;
    if release.draft || release.prerelease || !is_newer(&release.tag_name, current_version) {
        return Ok(None);
    }
    Ok(Some(UpdateInfo {
        version: release.tag_name.trim_start_matches(['v', 'V']).to_owned(),
        url: DOWNLOAD_PAGE.to_owned(),
        notes: release.body.filter(|b| !b.trim().is_empty()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tags() {
        assert_eq!(parse_version("v2.1.3"), Some((2, 1, 3)));
        assert_eq!(parse_version("2.1"), Some((2, 1, 0)));
        assert_eq!(parse_version("V3"), Some((3, 0, 0)));
        assert_eq!(parse_version("v2.1.0-beta.1"), None);
        assert_eq!(parse_version("latest"), None);
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer("v2.0.1", "2.0.0"));
        assert!(is_newer("v2.10.0", "2.9.9"));
        assert!(!is_newer("v2.0.0", "2.0.0"));
        assert!(!is_newer("v1.9.0", "2.0.0"));
        assert!(!is_newer("v3.0.0-rc1", "2.0.0"));
    }
}
