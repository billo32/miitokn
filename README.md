# MiITokn

**MiITokn** is a small macOS desktop app that extracts **Xiaomi / Mi Home device
tokens** from a local iPhone backup — no manual poking around in iBackup Viewer
or DB Browser.

Website & downloads: **[miitokn.com](https://miitokn.com)**

Tokens are what local controllers (Home Assistant, `python-miio`, etc.) need to
talk to your Xiaomi devices directly on your LAN. Mi Home stores them inside the
app's data, which ends up in your iPhone backup — this tool reads them out.

Built with [Tauri](https://tauri.app) (Rust backend + a dependency-free
HTML/CSS/JS frontend).

## Privacy

Everything runs **locally**. The app only reads backups already on your Mac,
works entirely offline, and never sends tokens or any data anywhere. There is no
network code in it at all.

## How it works

1. Finds local iPhone backups in `~/Library/Application Support/MobileSync/Backup/`.
2. Opens the selected backup (decrypting it with your backup password if it's encrypted).
3. Locates the Mi Home database (`*_mihome.sqlite`) inside the backup.
4. Reads each device's name, IP, and token (decrypting `ZTOKEN` where needed).

## Install (prebuilt)

Download the latest `.dmg` from **[miitokn.com](https://miitokn.com)** or the
[Releases](https://github.com/billo32/miitokn/releases) page,
open it, and drag the app to Applications.

The build is signed with an Apple Developer ID certificate and notarized by
Apple, so it opens like any other app — no Gatekeeper warnings or extra steps.

### Full Disk Access

On modern macOS the backup folder is protected. If the app shows “Folder access
required” or an empty backup list, grant it access in
**System Settings → Privacy & Security → Full Disk Access** (add the app itself,
not Terminal), then relaunch.

## Build from source

Requires a Mac (Xcode is needed to link and bundle the app).

```sh
# 1. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# restart your shell afterwards

# 2. Xcode command line tools
xcode-select --install

# 3. Tauri CLI
cargo install tauri-cli --version "^2.0.0" --locked

# 4. (once) generate the icon set from the source icon
cargo tauri icon icon.png

# 5. Build
cargo tauri build
```

The `.app` and `.dmg` land in `src-tauri/target/release/bundle/`.

To run in development:

```sh
cargo tauri dev
```

## Project structure

```
miitokn/
  icon.png                     source app icon (1024x1024)
  frontend/
    index.html                 the entire UI (no npm, no bundler)
  src-tauri/
    src/main.rs                Rust commands: check_access, list_backups,
                               extract_tokens, export_tokens
    tauri.conf.json
    capabilities/default.json
    Cargo.toml
  LICENSE                      GPL-3.0
  THIRD-PARTY-LICENSES.md      dependency licenses
```

## License

This project is licensed under the **GNU General Public License v3.0 or later** —
see [`LICENSE`](./LICENSE).

It links the [`crabapple`](https://github.com/ReagentX/crabapple) library
(GPL-3.0-or-later) for iOS backup decryption, which requires the whole
distributed application to be GPL-3.0. See
[`THIRD-PARTY-LICENSES.md`](./THIRD-PARTY-LICENSES.md) for the full dependency
breakdown.

## Disclaimer

Use this only with **your own local backups and your own devices**. It reads
backups that are already on your machine and does nothing over the network.

## Acknowledgments

This tool stands on the shoulders of the projects below — both the ones whose
code it builds on and the ones that figured out the hard parts first.

**Code used directly**

- [`crabapple`](https://github.com/ReagentX/crabapple) — Rust library that parses
  and decrypts iOS backups (Manifest keybag, per-file AES). Does the heavy lifting.
- [Tauri](https://tauri.app) — the Rust + WebView desktop app framework.
- [`rusqlite`](https://github.com/rusqlite/rusqlite) + bundled
  [SQLite](https://sqlite.org) — reading the Manifest and Mi Home databases.
- [RustCrypto](https://github.com/RustCrypto) crates (`aes`, `cbc`, `pbkdf2`,
  `aes-kw`, `hmac`, `sha1`/`sha2`) — the primitives behind backup and token decryption.
- [`rust-plist`](https://github.com/ebarnard/rust-plist) — parsing `Info.plist` /
  `Manifest.plist` / `Status.plist`.

**Inspiration & prior art**

- [`python-miio`](https://github.com/rytilahti/python-miio) — the reference for
  how Mi Home stores and encrypts device tokens (the `ZTOKEN` AES-ECB scheme).
- [Xiaomi-cloud-tokens-extractor](https://github.com/PiotrMachowski/Xiaomi-cloud-tokens-extractor)
  by Piotr Machowski — the well-known token extractor that inspired doing this
  locally from a backup instead of over the cloud.
- [Home Assistant](https://www.home-assistant.io/integrations/xiaomi_miio/)'s
  Xiaomi Miio integration — the reason these tokens are worth extracting.
- The many iOS backup format write-ups and tools (iBackup Viewer, DB Browser for
  SQLite) that make the manual version of this possible — this app just automates it.
