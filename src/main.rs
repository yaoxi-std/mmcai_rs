use std::env;
use std::io::Write;
use std::process::{self, Stdio};

use anyhow::{Context, Result};

mod args;
mod auth;
mod injector;
mod params;

fn main() -> Result<()> {
  let cli_args: Vec<String> = env::args().collect();
  let wrapper_args = args::parse_wrapper_args(&cli_args)?;

  let authlib_injector_path = injector::find_authlib_injector(None)
    .context("authlib-injector not found in the same directory as mmcai_rs")?;

  eprintln!(
    "[mmcai_rs] authlib-injector found at {:?}",
    authlib_injector_path
  );

  let mut minecraft_params = params::read_minecraft_params()?;
  let raw_username = params::extract_raw_username(&minecraft_params)?;
  let identity = args::parse_user_identity(&raw_username)?;

  eprintln!(
    "[mmcai_rs] Logging in as {} to {}",
    identity.username, identity.server_url
  );

  let client_token = auth::generate_client_token();

  // TODO: implement session caching in ~/.mmcai and password dialog
  let password = "";
  let login_result = auth::yggdrasil_login(
    &identity.username,
    password,
    &client_token,
    &identity.server_url,
  )?;

  eprintln!(
    "[mmcai_rs] Successfully authenticated as {}",
    login_result.selected_profile.name
  );

  params::modify_minecraft_params(
    &mut minecraft_params,
    &login_result.access_token,
    &login_result.selected_profile.id,
    &login_result.selected_profile.name,
  )?;

  let java_executable = env::var("INST_JAVA").context("INST_JAVA environment variable not set")?;

  let mut jvm_args = wrapper_args.jvm_args;
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
    format!(
      "-Dauthlibinjector.yggdrasil.prefetched={}",
      login_result.prefetched_data
    ),
  );

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
