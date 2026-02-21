use anyhow::{Context, Result};
use base64::prelude::*;
use reqwest::header;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthRequest<'a> {
  username: &'a str,
  password: &'a str,
  request_user: bool,
  client_token: &'a str,
  agent: Agent<'a>,
}

impl Default for AuthRequest<'_> {
  fn default() -> Self {
    AuthRequest {
      username: "",
      password: "",
      request_user: true,
      client_token: "",
      agent: Agent::default(),
    }
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Agent<'a> {
  name: &'a str,
  version: i32,
}

impl Default for Agent<'_> {
  fn default() -> Self {
    Agent {
      name: "Minecraft",
      version: 1,
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthResponse {
  access_token: String,
  selected_profile: Profile,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenPayload<'a> {
  access_token: &'a str,
  client_token: &'a str,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Profile {
  pub id: String,
  pub name: String,
}

#[derive(Debug)]
pub struct AuthResult {
  pub access_token: String,
  pub selected_profile: Profile,
}

fn build_http_client() -> Result<reqwest::blocking::Client> {
  reqwest::blocking::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .context("Failed to build HTTP client")
}

pub fn generate_client_token() -> String {
  Uuid::new_v4().to_string()
}

/// Fetch and base64-encode the Yggdrasil server metadata.
pub fn prefetch_server_data(api_url: &str) -> Result<String> {
  let client = build_http_client()?;
  let text = client
    .get(api_url)
    .send()
    .and_then(|r| r.text())
    .context("Cannot reach the authentication server")?;
  Ok(BASE64_STANDARD.encode(text))
}

/// Full authentication with username + password.
pub fn yggdrasil_authenticate(
  username: &str,
  password: &str,
  client_token: &str,
  api_url: &str,
) -> Result<AuthResult> {
  let client = build_http_client()?;

  let mut headers = header::HeaderMap::new();
  headers.insert("Content-Type", "application/json".parse().unwrap());

  let body = AuthRequest {
    username,
    password,
    client_token,
    ..AuthRequest::default()
  };

  let resp: AuthResponse = client
    .post(format!("{}/authserver/authenticate", api_url))
    .headers(headers)
    .json(&body)
    .send()
    .and_then(|r| r.json())
    .context("Authentication failed: wrong username or password")?;

  Ok(AuthResult {
    access_token: resp.access_token,
    selected_profile: resp.selected_profile,
  })
}

/// Check whether an existing access token is still valid (HTTP 204 = valid).
pub fn yggdrasil_validate(access_token: &str, client_token: &str, api_url: &str) -> Result<bool> {
  let client = build_http_client()?;
  let payload = TokenPayload {
    access_token,
    client_token,
  };

  let status = client
    .post(format!("{}/authserver/validate", api_url))
    .json(&payload)
    .send()
    .context("Cannot reach the authentication server")?
    .status();

  Ok(status.is_success())
}

/// Refresh an expired access token. Returns a new access token + profile.
pub fn yggdrasil_refresh(
  access_token: &str,
  client_token: &str,
  api_url: &str,
) -> Result<AuthResult> {
  let client = build_http_client()?;
  let payload = TokenPayload {
    access_token,
    client_token,
  };

  let resp: AuthResponse = client
    .post(format!("{}/authserver/refresh", api_url))
    .json(&payload)
    .send()
    .and_then(|r| r.json())
    .context("Token refresh failed")?;

  Ok(AuthResult {
    access_token: resp.access_token,
    selected_profile: resp.selected_profile,
  })
}
