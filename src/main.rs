use std::env;
use std::io::Write;
use std::process::{self, Stdio};

use anyhow::{Context, Result};

use args::UserIdentity;
use auth::AuthResult;
use session::SessionData;

mod args;
mod auth;
mod dialog;
mod injector;
mod params;
mod session;

/// Try cached session → validate → refresh → prompt password → authenticate.
fn resolve_auth(identity: &UserIdentity) -> Result<(AuthResult, String)> {
  if let Some(cached) = session::load_session(&identity.username, &identity.server_url)? {
    eprintln!("[mmcai_rs] Found cached session, validating...");

    if auth::yggdrasil_validate(
      &cached.access_token,
      &cached.client_token,
      &identity.server_url,
    )? {
      eprintln!("[mmcai_rs] Session is valid");
      let profile = cached.profile();
      let result = AuthResult {
        access_token: cached.access_token,
        selected_profile: profile,
      };
      return Ok((result, cached.client_token));
    }

    eprintln!("[mmcai_rs] Session expired, trying refresh...");
    match auth::yggdrasil_refresh(
      &cached.access_token,
      &cached.client_token,
      &identity.server_url,
    ) {
      Ok(result) => {
        save_session_from_result(identity, &result, &cached.client_token)?;
        eprintln!("[mmcai_rs] Session refreshed");
        return Ok((result, cached.client_token));
      }
      Err(e) => {
        eprintln!("[mmcai_rs] Refresh failed: {:#}", e);
      }
    }
  }

  eprintln!("[mmcai_rs] Requesting password...");
  let password = dialog::prompt_password(&identity.username, &identity.server_url)?;
  let client_token = auth::generate_client_token();
  let result = auth::yggdrasil_authenticate(
    &identity.username,
    &password,
    &client_token,
    &identity.server_url,
  )?;
  save_session_from_result(identity, &result, &client_token)?;
  Ok((result, client_token))
}

fn save_session_from_result(
  identity: &UserIdentity,
  result: &AuthResult,
  client_token: &str,
) -> Result<()> {
  let data = SessionData {
    access_token: result.access_token.clone(),
    client_token: client_token.to_string(),
    profile_id: result.selected_profile.id.clone(),
    profile_name: result.selected_profile.name.clone(),
  };
  session::save_session(&identity.username, &identity.server_url, &data)
}

fn main() -> Result<()> {
  let cli_args: Vec<String> = env::args().collect();
  let wrapper_args = args::parse_wrapper_args(&cli_args)?;

  let mut minecraft_params = params::read_minecraft_params()?;
  let raw_username = params::extract_raw_username(&minecraft_params)?;

  let mut jvm_args = wrapper_args.jvm_args;

  match args::parse_user_identity(&raw_username) {
    Ok(mut identity) => {
      let authlib_injector_path = injector::find_authlib_injector(None)
        .context("authlib-injector not found in the same directory as mmcai_rs")?;

      eprintln!(
        "[mmcai_rs] authlib-injector found at {:?}",
        authlib_injector_path
      );

      eprintln!(
        "[mmcai_rs] Resolving API URL for {}...",
        identity.server_url
      );

      let resolved_url = auth::resolve_api_url(&identity.server_url)?;
      if resolved_url != identity.server_url {
        eprintln!(
          "[mmcai_rs] ALI: {} -> {}",
          identity.server_url, resolved_url
        );
        identity.server_url = resolved_url;
      }

      eprintln!(
        "[mmcai_rs] Logging in as {} to {}",
        identity.username, identity.server_url
      );

      let prefetched_data = auth::prefetch_server_data(&identity.server_url)?;
      let (auth_result, _client_token) = resolve_auth(&identity)?;

      eprintln!(
        "[mmcai_rs] Successfully authenticated as {}",
        auth_result.selected_profile.name
      );

      params::modify_minecraft_params(
        &mut minecraft_params,
        &auth_result.access_token,
        &auth_result.selected_profile.id,
        &auth_result.selected_profile.name,
      )?;

      jvm_args.insert(
        0,
        format!(
          "-javaagent:{}={}",
          authlib_injector_path
            .to_str()
            .context("Invalid authlib-injector path")?,
          identity.server_url
        ),
      );
      jvm_args.insert(
        1,
        format!("-Dauthlibinjector.yggdrasil.prefetched={}", prefetched_data),
      );
    }
    Err(_) => {
      eprintln!(
        "[mmcai_rs] '{}' is not in <user>@<server> format, \
         bypassing authlib-injector (passthrough mode)",
        raw_username
      );
    }
  }

  let java_executable = env::var("INST_JAVA").context("INST_JAVA environment variable not set")?;

  #[cfg(debug_assertions)]
  {
    eprintln!("[mmcai_rs] java_executable: {:?}", java_executable);
    eprintln!("[mmcai_rs] jvm_args: {:?}", jvm_args);
    eprintln!("[mmcai_rs] minecraft_params: {:?}", minecraft_params);
  }

  let mut child = process::Command::new(java_executable)
    .args(jvm_args)
    .stdin(Stdio::piped())
    .stdout(Stdio::inherit())
    .spawn()
    .context("Failed to start Minecraft process")?;

  let stdin = child.stdin.as_mut().context("Child stdin unavailable")?;
  for line in &minecraft_params {
    writeln!(stdin, "{}", line).context("Failed to write minecraft params to child stdin")?;
  }

  let status = child
    .wait()
    .context("Failed to wait for Minecraft process")?;
  if !status.success() {
    process::exit(status.code().unwrap_or(1));
  }

  Ok(())
}
