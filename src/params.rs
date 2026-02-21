use std::io::{self, BufRead};

use anyhow::{bail, Context, Result};

/// Read minecraft launch parameters from stdin until "launch" line.
pub fn read_minecraft_params() -> Result<Vec<String>> {
  let stdin = io::stdin();
  let mut params = Vec::new();
  for line in stdin.lock().lines() {
    let line = line
      .context("Failed to read minecraft params from stdin")?
      .trim()
      .to_string();
    let is_launch = line == "launch";
    params.push(line);
    if is_launch {
      break;
    }
  }
  Ok(params)
}

/// Extract the raw username value from the `param --username` / `param <value>` pair.
pub fn extract_raw_username(params: &[String]) -> Result<String> {
  for i in 0..params.len() {
    if params[i].contains("param --username") {
      let next = params
        .get(i + 1)
        .context("Missing value after 'param --username'")?;
      let value = next
        .strip_prefix("param ")
        .context("Expected 'param <value>' after 'param --username'")?;
      return Ok(value.to_string());
    }
  }
  bail!("'param --username' not found in minecraft params")
}

/// Replace authentication-related fields in the minecraft params.
pub fn modify_minecraft_params(
  minecraft_params: &mut [String],
  access_token: &str,
  uuid: &str,
  playername: &str,
) -> Result<()> {
  for index in 0..minecraft_params.len() {
    match minecraft_params[index].as_str() {
      line if line.contains("param --username") => {
        let next = minecraft_params
          .get_mut(index + 1)
          .context("Missing value after 'param --username'")?;
        *next = format!("param {}", playername);
      }
      line if line.contains("param --uuid") => {
        let next = minecraft_params
          .get_mut(index + 1)
          .context("Missing value after 'param --uuid'")?;
        *next = format!("param {}", uuid);
      }
      line if line.contains("param --accessToken") => {
        let next = minecraft_params
          .get_mut(index + 1)
          .context("Missing value after 'param --accessToken'")?;
        *next = format!("param {}", access_token);
      }
      line if line.contains("userName ") => {
        minecraft_params[index] = format!("userName {}", playername);
      }
      line if line.contains("sessionId ") => {
        minecraft_params[index] = format!("sessionId token:{}", access_token);
      }
      _ => {}
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_raw_username() {
    let params = vec![
      "param --username".to_string(),
      "param player@https://skin.example.com/api/yggdrasil".to_string(),
      "param --uuid".to_string(),
      "param 00000000-0000-0000-0000-000000000000".to_string(),
      "launch".to_string(),
    ];
    let username = extract_raw_username(&params).unwrap();
    assert_eq!(username, "player@https://skin.example.com/api/yggdrasil");
  }

  #[test]
  fn test_extract_raw_username_not_found() {
    let params = vec![
      "param --uuid".to_string(),
      "param 00000000-0000-0000-0000-000000000000".to_string(),
      "launch".to_string(),
    ];
    assert!(extract_raw_username(&params).is_err());
  }

  #[test]
  fn test_extract_raw_username_missing_value() {
    let params = vec!["param --username".to_string()];
    assert!(extract_raw_username(&params).is_err());
  }

  #[test]
  fn test_modify_minecraft_params() {
    let mut params = vec![
      "---START---".to_string(),
      "param --username".to_string(),
      "param AnyHow".to_string(),
      "param --uuid".to_string(),
      "param AnyHow".to_string(),
      "param --accessToken".to_string(),
      "param AnyHow".to_string(),
      "userName AnyHow".to_string(),
      "sessionId AnyHow".to_string(),
      "launch".to_string(),
      "---END---".to_string(),
    ];
    modify_minecraft_params(&mut params, "TOKEN_123", "UUID_456", "Steve").unwrap();
    assert_eq!(
      params,
      vec![
        "---START---",
        "param --username",
        "param Steve",
        "param --uuid",
        "param UUID_456",
        "param --accessToken",
        "param TOKEN_123",
        "userName Steve",
        "sessionId token:TOKEN_123",
        "launch",
        "---END---",
      ]
    );
  }

  #[test]
  fn test_modify_minecraft_params_partial() {
    let mut params = vec![
      "param --username".to_string(),
      "param OldName".to_string(),
      "launch".to_string(),
    ];
    modify_minecraft_params(&mut params, "tok", "uid", "NewName").unwrap();
    assert_eq!(params[1], "param NewName");
    assert_eq!(params[2], "launch");
  }
}
