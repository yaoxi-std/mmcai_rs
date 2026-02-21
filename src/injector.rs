use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::{env, fs};

/// Search for an `authlib-injector-*.jar` file in the given directory,
/// or in the same directory as the current executable if `path` is `None`.
pub fn find_authlib_injector(path: Option<&Path>) -> Option<PathBuf> {
  let path = match path {
    Some(p) => p.to_path_buf(),
    None => {
      let exe_path = env::current_exe().ok()?;
      exe_path.parent()?.to_path_buf()
    }
  };

  let is_filename_valid =
    |filename: &str| filename.starts_with("authlib-injector") && filename.ends_with(".jar");

  fs::read_dir(path).ok().and_then(|entries| {
    entries
      .filter_map(IoResult::ok)
      .find(|entry| {
        let file_name = entry.file_name();
        file_name.to_str().map_or(false, is_filename_valid)
      })
      .map(|entry| entry.path())
  })
}

#[cfg(test)]
mod tests {
  use assert_fs::prelude::{FileTouch, PathChild};

  use super::*;

  #[test]
  fn test_find_authlib_injector() {
    let cases = [
      ("authlib-injector-1.0.0.jar", true),
      ("authlib-injector-1.0.0.zip", false),
      ("authlib-injector-1.0.0", false),
      ("authlib-injector-.catch.me.if.you.can.jar", true),
      ("not-start-with.authlib-injector.jar", false),
      ("authlib-injector.jar.not-end-with", false),
    ];

    for (filename, should_exist) in cases {
      let temp_dir = assert_fs::TempDir::new().unwrap();
      let input_file = temp_dir.child(filename);
      input_file.touch().unwrap();
      if should_exist {
        assert_eq!(
          find_authlib_injector(Some(&temp_dir)).unwrap(),
          input_file.path(),
          "Expected to find {}",
          filename
        );
      } else {
        assert!(
          find_authlib_injector(Some(&temp_dir)).is_none(),
          "Expected NOT to find {}",
          filename
        );
      }
      temp_dir.close().unwrap();
    }
  }
}
