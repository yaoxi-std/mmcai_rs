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

#[derive(Deserialize, Debug)]
pub struct Profile {
  pub id: String,
  pub name: String,
}

#[derive(Debug)]
pub struct LoginResult {
  pub prefetched_data: String,
  pub access_token: String,
  pub selected_profile: Profile,
}

pub fn generate_client_token() -> String {
  Uuid::new_v4().to_string()
}

pub fn yggdrasil_login(
  username: &str,
  password: &str,
  client_token: &str,
  api_url: &str,
) -> Result<LoginResult> {
  let client = reqwest::blocking::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .context("Failed to build HTTP client")?;

  let prefetched_data_text = client
    .get(api_url)
    .send()
    .and_then(|r| r.text())
    .context("Cannot reach the authentication server")?;
  let prefetched_data = BASE64_STANDARD.encode(prefetched_data_text);

  let mut headers = header::HeaderMap::new();
  headers.insert("Content-Type", "application/json".parse().unwrap());

  let body = AuthRequest {
    username,
    password,
    client_token,
    ..AuthRequest::default()
  };

  let auth_response: AuthResponse = client
    .post(format!("{}/authserver/authenticate", api_url))
    .headers(headers)
    .json(&body)
    .send()
    .and_then(|r| r.json())
    .context("Authentication failed: wrong username or password")?;

  Ok(LoginResult {
    prefetched_data,
    access_token: auth_response.access_token,
    selected_profile: auth_response.selected_profile,
  })
}
