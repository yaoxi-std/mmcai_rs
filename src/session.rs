use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::auth::Profile;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionData {
  pub access_token: String,
  pub client_token: String,
  pub profile_id: String,
  pub profile_name: String,
}

impl SessionData {
  pub fn profile(&self) -> Profile {
    Profile {
      id: self.profile_id.clone(),
      name: self.profile_name.clone(),
    }
  }
}

/// The sessions file lives alongside the executable (same dir as authlib-injector).
fn sessions_file_path() -> Result<PathBuf> {
  let exe_path = env::current_exe().context("Cannot determine executable path")?;
  let exe_dir = exe_path
    .parent()
    .context("Cannot determine executable directory")?;
  Ok(exe_dir.join("mmcai_sessions.json"))
}

type SessionMap = HashMap<String, SessionData>;

fn session_key(username: &str, server_url: &str) -> String {
  format!("{}@{}", username, server_url)
}

fn load_all() -> Result<SessionMap> {
  let path = sessions_file_path()?;
  if !path.exists() {
    return Ok(SessionMap::new());
  }
  let content = fs::read_to_string(&path).context("Failed to read sessions file")?;
  serde_json::from_str(&content).context("Failed to parse sessions file")
}

fn save_all(sessions: &SessionMap) -> Result<()> {
  let path = sessions_file_path()?;
  let content = serde_json::to_string_pretty(sessions).context("Failed to serialize sessions")?;
  fs::write(&path, content).context("Failed to write sessions file")
}

pub fn load_session(username: &str, server_url: &str) -> Result<Option<SessionData>> {
  let sessions = load_all()?;
  Ok(sessions.get(&session_key(username, server_url)).cloned())
}

pub fn save_session(username: &str, server_url: &str, data: &SessionData) -> Result<()> {
  let mut sessions = load_all().unwrap_or_default();
  sessions.insert(session_key(username, server_url), data.clone());
  save_all(&sessions)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_session_key() {
    assert_eq!(
      session_key("player", "https://example.com"),
      "player@https://example.com"
    );
  }

  #[test]
  fn test_session_data_profile() {
    let data = SessionData {
      access_token: "tok".into(),
      client_token: "ct".into(),
      profile_id: "id-123".into(),
      profile_name: "Steve".into(),
    };
    let p = data.profile();
    assert_eq!(p.id, "id-123");
    assert_eq!(p.name, "Steve");
  }

  #[test]
  fn test_session_roundtrip_json() {
    let data = SessionData {
      access_token: "a".into(),
      client_token: "c".into(),
      profile_id: "p".into(),
      profile_name: "n".into(),
    };

    let mut map = SessionMap::new();
    map.insert(session_key("user", "https://example.com"), data.clone());

    let json = serde_json::to_string(&map).unwrap();
    let parsed: SessionMap = serde_json::from_str(&json).unwrap();

    let got = parsed.get("user@https://example.com").unwrap();
    assert_eq!(got.access_token, "a");
    assert_eq!(got.client_token, "c");
    assert_eq!(got.profile_id, "p");
    assert_eq!(got.profile_name, "n");
  }
}
