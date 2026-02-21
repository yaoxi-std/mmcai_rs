use std::process::Command;

use anyhow::{bail, Context, Result};

/// Prompt the user for a password using a native system dialog.
///
/// The dialog shows the username and server URL for context.
/// Returns `Err` if the user cancels or the dialog tool is unavailable.
pub fn prompt_password(username: &str, server_url: &str) -> Result<String> {
  prompt_password_impl(username, server_url)
}

#[cfg(target_os = "macos")]
fn prompt_password_impl(username: &str, server_url: &str) -> Result<String> {
  let output = Command::new("osascript")
    .args(["-e", r#"set u to (system attribute "MMCAI_USER")"#])
    .args(["-e", r#"set s to (system attribute "MMCAI_URL")"#])
    .args([
      "-e",
      r#"set p to "Enter password for " & u & return & return & "Server: " & s"#,
    ])
    .args([
      "-e",
      r#"text returned of (display dialog p default answer "" with hidden answer buttons {"Cancel", "OK"} default button "OK" with title "mmcai_rs")"#,
    ])
    .env("MMCAI_USER", username)
    .env("MMCAI_URL", server_url)
    .output()
    .context("Failed to launch osascript")?;

  if !output.status.success() {
    bail!("Password dialog was cancelled");
  }

  let password = String::from_utf8(output.stdout)
    .context("Invalid UTF-8 in dialog output")?
    .trim()
    .to_string();
  Ok(password)
}

#[cfg(target_os = "linux")]
fn prompt_password_impl(username: &str, server_url: &str) -> Result<String> {
  let prompt = format!("Enter password for {}\nServer: {}", username, server_url);

  // Try zenity first (GTK)
  if let Ok(output) = Command::new("zenity")
    .args([
      "--entry",
      "--hide-text",
      "--title=mmcai_rs",
      &format!("--text={}", prompt),
    ])
    .output()
  {
    if output.status.success() {
      return Ok(
        String::from_utf8(output.stdout)
          .context("Invalid UTF-8 in zenity output")?
          .trim()
          .to_string(),
      );
    }
    if output.status.code() == Some(1) {
      bail!("Password dialog was cancelled");
    }
  }

  // Fallback to kdialog (KDE)
  let output = Command::new("kdialog")
    .args(["--password", &prompt, "--title", "mmcai_rs"])
    .output()
    .context(
      "Neither zenity nor kdialog found.\n\
       Install zenity (GTK) or kdialog (KDE) for the password dialog.",
    )?;

  if !output.status.success() {
    bail!("Password dialog was cancelled");
  }

  Ok(
    String::from_utf8(output.stdout)
      .context("Invalid UTF-8 in kdialog output")?
      .trim()
      .to_string(),
  )
}

#[cfg(target_os = "windows")]
fn prompt_password_impl(username: &str, server_url: &str) -> Result<String> {
  let script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$u = $env:MMCAI_USER
$s = $env:MMCAI_URL
$f = New-Object Windows.Forms.Form -Property @{
  Text='mmcai_rs'
  Size='420,200'
  StartPosition='CenterScreen'
  FormBorderStyle='FixedDialog'
  MaximizeBox=$false
  MinimizeBox=$false
  TopMost=$true
}
$l = New-Object Windows.Forms.Label -Property @{
  Text="Enter password for $u`r`nServer: $s"
  Location='15,15'
  Size='375,45'
  Font='Microsoft Sans Serif,9'
}
$f.Controls.Add($l)
$t = New-Object Windows.Forms.TextBox -Property @{
  Location='15,65'
  Size='375,25'
  UseSystemPasswordChar=$true
}
$f.Controls.Add($t)
$ok = New-Object Windows.Forms.Button -Property @{
  Text='OK'
  Location='225,110'
  Size='80,30'
  DialogResult='OK'
}
$f.AcceptButton = $ok
$f.Controls.Add($ok)
$ca = New-Object Windows.Forms.Button -Property @{
  Text='Cancel'
  Location='310,110'
  Size='80,30'
  DialogResult='Cancel'
}
$f.CancelButton = $ca
$f.Controls.Add($ca)
$f.Add_Shown({$t.Focus()})
if ($f.ShowDialog() -eq 'OK') {
  [Console]::Write($t.Text)
} else {
  exit 1
}
"#;

  let output = Command::new("powershell")
    .args(["-NoProfile", "-Command", script])
    .env("MMCAI_USER", username)
    .env("MMCAI_URL", server_url)
    .output()
    .context("Failed to launch PowerShell password dialog")?;

  if !output.status.success() {
    bail!("Password dialog was cancelled");
  }

  Ok(
    String::from_utf8(output.stdout)
      .context("Invalid UTF-8 in PowerShell output")?
      .trim()
      .to_string(),
  )
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn prompt_password_impl(_username: &str, _server_url: &str) -> Result<String> {
  bail!("Password dialog is not supported on this platform")
}
