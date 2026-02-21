# mmcai_rs

中文版请见[这里](README_zh.md)。

_Tee Hee._

Prism Launcher/MultiMC itself does not support authlib-injector (custom/homebrew/alternative/pirate/whatever you call it/... Yggdrasil servers) and officially says it never will. So I made this.

This project is inspired by [mmcai.sh](https://github.com/baobao1270/mmcai.sh), but it only supports Linux and macOS. I want to make it work on Windows.

Windows, macOS, Linux, all supported. No plaintext passwords. Sessions are cached. Microsoft accounts work as-is.

## How to use

1. Download mmcai_rs from [Releases](https://github.com/CatMe0w/mmcai_rs/releases) and authlib-injector from [here](https://github.com/yushijinhun/authlib-injector/releases).

2. It is recommended to put both files under `~/.mmcai` (create the directory if needed):

   ```
   ~/.mmcai/
   ├── mmcai_rs          (or mmcai_rs.exe on Windows)
   └── authlib-injector-X.Y.Z.jar
   ```

3. In Prism Launcher, create an **offline account** with the name `<username>@<server>`:

   - `<username>` is your account name on the Yggdrasil server.
   - `<server>` is the server domain or full API URL.
   - Examples: `player@littleskin.cn`, `player@https://example.com/api/yggdrasil`

   ![Edit offline username in Prism](assets/username.png)

4. Edit an instance, go to **Settings > Custom commands**, and set the **Wrapper command** to the absolute path of mmcai_rs:

   ```
   /home/you/.mmcai/mmcai_rs
   ```

5. Launch the game. A system dialog will ask for your password on first login. The session is cached afterwards.

> **Tip:** Microsoft accounts work without any extra setup. If the active account is a Microsoft account, mmcai_rs detects it automatically and passes through without injecting authlib-injector.

## License

[MIT License](https://opensource.org/licenses/MIT)

Exception: The file `easteregg.jpg` is all rights reserved. You may not use it without permission. Credits to [ZH9c418](https://github.com/zh9c418) & [瑞狩](https://twitter.com/Ruishou_Nyako).
