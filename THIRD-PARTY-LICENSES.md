# Third-party licenses

MiITokn is distributed under the **GNU General Public License
v3.0 or later** (see [`LICENSE`](./LICENSE)). It is built on open-source
components. This file summarizes their licenses.

## Why the whole app is GPL-3.0

The core backup-decryption library **[`crabapple`](https://github.com/ReagentX/crabapple)**
is licensed under **GPL-3.0-or-later** and is statically linked into the
application binary. Under the GPL, the combined distributed work must therefore
be released under GPL-3.0-or-later. That is the reason this project — including
the prebuilt binaries — carries the GPL-3.0 license rather than a permissive one.

## Direct dependencies (Rust)

| Crate | License |
| --- | --- |
| crabapple | **GPL-3.0-or-later** |
| tauri | Apache-2.0 OR MIT |
| tauri-build | Apache-2.0 OR MIT |
| tauri-plugin-dialog | Apache-2.0 OR MIT |
| rusqlite | MIT |
| plist | MIT |
| aes | MIT OR Apache-2.0 |
| serde | MIT OR Apache-2.0 |
| serde_json | MIT OR Apache-2.0 |
| hex | MIT OR Apache-2.0 |
| tempfile | MIT OR Apache-2.0 |
| dirs | MIT OR Apache-2.0 |

## Transitive dependencies

The full dependency graph (~475 additional transitive crates) is, apart from
`crabapple`, made up of permissive licenses. The distribution across the whole
graph is:

- The vast majority: `MIT`, `Apache-2.0`, or the dual `MIT OR Apache-2.0`
- Also present: `BSD-3-Clause`, `Zlib`, `ISC`, `Unicode-3.0`, `Unlicense`,
  `0BSD`, `CC0-1.0`
- `MPL-2.0` (a handful of crates): file-level copyleft. These crates are used
  unmodified; their source is publicly available on <https://crates.io>.

To regenerate an exact, per-crate report:

```sh
cargo install cargo-about        # or: cargo install cargo-license
cd src-tauri
cargo about generate about.hbs   # or: cargo license
```

## Bundled SQLite

`rusqlite` is built with the `bundled` feature, which compiles **SQLite**
(<https://sqlite.org>). SQLite is released into the **public domain**.

## Runtime engines

Tauri renders the UI in the operating system's WebView. On macOS this is Apple's
**WKWebView** (system component). No JavaScript frameworks are bundled — the
frontend is dependency-free vanilla HTML/CSS/JS.
