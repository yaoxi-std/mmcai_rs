use anyhow::{bail, Context, Result};

/// Parsed wrapper command-line arguments.
///
/// When Prism calls the wrapper, the command line is:
///   `<wrapper_path> <java_path> [jvm_args...]`
///
/// We use the `INST_JAVA` env var for the java executable,
/// so `java_path` (args[1]) is skipped.
pub struct WrapperArgs {
  pub jvm_args: Vec<String>,
}

/// A user identity parsed from the `<username>@<server-url>` format
/// used as the offline account name in Prism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserIdentity {
  pub username: String,
  pub server_url: String,
}

pub fn parse_wrapper_args(args: &[String]) -> Result<WrapperArgs> {
  if args.len() < 2 {
    bail!(
      "Usage: set this binary as the wrapper command in Prism Launcher.\n\
       Create an offline account named <username>@<server-url> in Prism."
    );
  }
  // args[0] = wrapper path
  // args[1] = java path (from Prism, we use INST_JAVA instead)
  // args[2..] = JVM args
  Ok(WrapperArgs {
    jvm_args: args.get(2..).unwrap_or_default().to_vec(),
  })
}

/// Parse `<username>@<server-url>` into its components.
///
/// The server URL must start with `http://` or `https://`.
/// The split happens at the last `@http://` or `@https://` boundary,
/// so usernames containing `@` (e.g. email addresses) are supported.
pub fn parse_user_identity(raw_username: &str) -> Result<UserIdentity> {
  let separator_pos = raw_username
    .rfind("@https://")
    .or_else(|| raw_username.rfind("@http://"))
    .with_context(|| {
      format!(
        "Invalid username format '{}': expected <username>@<server-url>",
        raw_username
      )
    })?;

  let username = &raw_username[..separator_pos];
  let server_url = &raw_username[separator_pos + 1..];

  if username.is_empty() {
    bail!(
      "Username part is empty in '{}': expected <username>@<server-url>",
      raw_username
    );
  }

  Ok(UserIdentity {
    username: username.to_string(),
    server_url: server_url.to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_wrapper_args_too_few() {
    let args = vec!["mmcai_rs".to_string()];
    assert!(parse_wrapper_args(&args).is_err());
  }

  #[test]
  fn test_parse_wrapper_args_minimal() {
    let args = vec!["mmcai_rs".to_string(), "/usr/bin/java".to_string()];
    let result = parse_wrapper_args(&args).unwrap();
    assert!(result.jvm_args.is_empty());
  }

  #[test]
  fn test_parse_wrapper_args_with_jvm_args() {
    let args = vec![
      "mmcai_rs".to_string(),
      "/usr/bin/java".to_string(),
      "-Xmx2G".to_string(),
      "-Xms512M".to_string(),
    ];
    let result = parse_wrapper_args(&args).unwrap();
    assert_eq!(result.jvm_args, vec!["-Xmx2G", "-Xms512M"]);
  }

  #[test]
  fn test_parse_user_identity_https() {
    let id = parse_user_identity("player@https://skin.example.com/api/yggdrasil").unwrap();
    assert_eq!(id.username, "player");
    assert_eq!(id.server_url, "https://skin.example.com/api/yggdrasil");
  }

  #[test]
  fn test_parse_user_identity_http() {
    let id = parse_user_identity("player@http://localhost:8080").unwrap();
    assert_eq!(id.username, "player");
    assert_eq!(id.server_url, "http://localhost:8080");
  }

  #[test]
  fn test_parse_user_identity_email_username() {
    let id =
      parse_user_identity("user@example.com@https://skin.example.com/api/yggdrasil").unwrap();
    assert_eq!(id.username, "user@example.com");
    assert_eq!(id.server_url, "https://skin.example.com/api/yggdrasil");
  }

  #[test]
  fn test_parse_user_identity_no_server_url() {
    assert!(parse_user_identity("player").is_err());
  }

  #[test]
  fn test_parse_user_identity_empty_username() {
    assert!(parse_user_identity("@https://example.com").is_err());
  }

  #[test]
  fn test_parse_user_identity_no_scheme() {
    assert!(parse_user_identity("player@example.com").is_err());
  }
}
