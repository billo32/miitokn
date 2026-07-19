// MiITokn — extract Xiaomi / Mi Home device tokens from a local iPhone backup.
// Copyright (C) 2026 billo32
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crabapple::backup::device::get_device_basic_info;
use crabapple::{Authentication, Backup};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const MIHOME_DOMAIN: &str = "AppDomain-com.xiaomi.mihome";

#[derive(Serialize)]
struct BackupInfo {
    udid: String,
    name: String,
    ios: String,
    /// Last backup time as Unix epoch seconds (string), or "" if unknown.
    /// Formatting into a localized date is done on the frontend.
    last_backup_date: String,
    is_encrypted: bool,
}

/// Progress update streamed to the frontend during `extract_tokens`.
#[derive(Clone, Serialize)]
struct Progress {
    percent: u8,
    step: String,
    detail: String,
}

fn emit_progress(app: &AppHandle, percent: u8, step: &str, detail: &str) {
    let _ = app.emit(
        "extraction-progress",
        Progress {
            percent,
            step: step.to_string(),
            detail: detail.to_string(),
        },
    );
}

/// Reads `IsEncrypted` from `<backup>/Manifest.plist` without needing a password.
fn read_is_encrypted(dir: &Path) -> bool {
    plist::Value::from_file(dir.join("Manifest.plist"))
        .ok()
        .and_then(|v| v.into_dictionary())
        .and_then(|d| d.get("IsEncrypted").and_then(plist::Value::as_boolean))
        .unwrap_or(false)
}

/// Last backup timestamp as Unix epoch seconds. Prefers `Status.plist` (`Date`),
/// falls back to `Info.plist` (`Last Backup Date`). Returns "" when unavailable.
fn read_last_backup_date(dir: &Path) -> String {
    for (file, key) in [("Status.plist", "Date"), ("Info.plist", "Last Backup Date")] {
        if let Ok(v) = plist::Value::from_file(dir.join(file)) {
            if let Some(date) = v.as_dictionary().and_then(|d| d.get(key)).and_then(plist::Value::as_date) {
                if let Ok(secs) = std::time::SystemTime::from(date).duration_since(UNIX_EPOCH) {
                    return secs.as_secs().to_string();
                }
            }
        }
    }
    String::new()
}

#[derive(Serialize, Default)]
struct XiaomiDevice {
    name: String,
    model: String,
    ip: String,
    mac: String,
    token: String,
}

fn backup_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|h| h.join("Library/Application Support/MobileSync/Backup"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir().map(|d| d.join("Apple Computer").join("MobileSync").join("Backup"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// Decrypts ZTOKEN the same way python-miio does for newer Mi Home iOS
/// databases: AES-128-ECB with an all-zero key, applied to the first 64 hex
/// characters (32 bytes / 2 blocks) of the stored value. Older app versions
/// store the token as plain 32 hex characters already.
fn decrypt_ztoken(ztoken: &str) -> String {
    if ztoken.is_empty() || ztoken.len() <= 32 {
        return ztoken.to_string();
    }

    use aes::cipher::{Array, BlockCipherDecrypt, KeyInit};
    use aes::Aes128;

    let ciphertext = match hex::decode(&ztoken[..64]) {
        Ok(bytes) => bytes,
        Err(_) => return ztoken.to_string(),
    };

    let key = Array::from([0u8; 16]);
    let cipher = Aes128::new(&key);

    let mut output = Vec::with_capacity(ciphertext.len());
    for chunk in ciphertext.chunks(16) {
        let Ok(chunk16): Result<[u8; 16], _> = chunk.try_into() else {
            break;
        };
        let mut block = Array::from(chunk16);
        cipher.decrypt_block(&mut block);
        output.extend_from_slice(&block);
    }

    // The plaintext is the token as an ASCII hex string, optionally followed by
    // block-cipher padding (PKCS7 => 0x01..=0x10, or zero padding). Hex digits
    // are 0x30..=0x66, so none of those padding bytes collide with the token —
    // keeping the leading hex run strips the padding cleanly. Trimming only NUL
    // bytes (as before) left PKCS7 padding like 0x08 in the string, which
    // rendered as control characters (e.g. backspaces).
    String::from_utf8_lossy(&output)
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect()
}

/// Whether the app can read the iPhone backup folder. On modern macOS the
/// `MobileSync` directory is TCC-protected and reading it requires Full Disk
/// Access. If nothing has been created yet, there is no restriction to hit.
#[tauri::command]
fn check_access() -> bool {
    let Some(root) = backup_root() else {
        return false;
    };
    if root.exists() {
        return std::fs::read_dir(&root).is_ok();
    }
    match root.parent() {
        Some(mobile_sync) if mobile_sync.exists() => std::fs::read_dir(mobile_sync).is_ok(),
        _ => true,
    }
}

/// Opens the Full Disk Access pane in System Settings so the user can grant
/// access to the backup folder.
#[tauri::command]
fn request_access() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let root = backup_root().ok_or_else(|| "Backup folder not found".to_string())?;
    if !root.is_dir() {
        // Folder not created yet — no backups rather than an error.
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    let read_dir = std::fs::read_dir(&root).map_err(|e| e.to_string())?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Ok(info) = get_device_basic_info(&path) {
            result.push(BackupInfo {
                udid: info.unique_device_id,
                name: info.device_name,
                ios: info.product_version,
                last_backup_date: read_last_backup_date(&path),
                is_encrypted: read_is_encrypted(&path),
            });
        }
    }

    // Newest backup first.
    result.sort_by(|a, b| b.last_backup_date.cmp(&a.last_backup_date));

    Ok(result)
}

// Async command so the heavy work runs off the main thread; a synchronous
// command would block the UI thread, freezing the window and starving the
// progress events until the whole operation finished.
#[tauri::command]
async fn extract_tokens(
    app: AppHandle,
    udid: String,
    password: String,
) -> Result<Vec<XiaomiDevice>, String> {
    tauri::async_runtime::spawn_blocking(move || extract_tokens_blocking(app, udid, password))
        .await
        .map_err(|e| format!("Execution error: {e}"))?
}

fn extract_tokens_blocking(
    app: AppHandle,
    udid: String,
    password: String,
) -> Result<Vec<XiaomiDevice>, String> {
    let root = backup_root().ok_or_else(|| "Backup folder not found".to_string())?;
    let udid_folder = root.join(&udid);

    emit_progress(&app, 5, "Opening backup…", "Manifest.plist");

    // Empty password means the backup is not encrypted (Authentication::None).
    let auth = if password.is_empty() {
        Authentication::None
    } else {
        Authentication::Password(password)
    };
    // `PASSWORD:` prefix tells the frontend to return to the password step.
    let backup = Backup::open(&udid_folder, &auth)
        .map_err(|e| format!("PASSWORD:Couldn't open the backup (check the password): {e}"))?;

    emit_progress(&app, 40, "Reading backup index…", "Manifest.db");

    // Ищем нужный файл напрямую в Manifest.db по domain/relativePath, не через
    // backup.entries(): тот метод разбирает NSKeyedArchiver-метаданные ВСЕХ
    // файлов бэкапа (фото, сообщения и т.д.), и на нестандартной записи где-то
    // в другом домене может упасть с ошибкой парсинга plist, даже если нужный
    // нам файл Mi Home в полном порядке.
    emit_progress(&app, 68, "Locating Mi Home data…", "com.xiaomi.mihome");

    let manifest_conn = rusqlite::Connection::open(backup.manifest_db_path())
        .map_err(|e| format!("Couldn't open Manifest.db: {e}"))?;
    let mut mstmt = manifest_conn
        .prepare("SELECT fileID, relativePath FROM Files WHERE domain = ?1 AND relativePath LIKE 'Documents/%'")
        .map_err(|e| e.to_string())?;
    let file_ids: Vec<(String, String)> = mstmt
        .query_map([MIHOME_DOMAIN], |row| {
            let file_id: String = row.get(0)?;
            let relative_path: String = row.get(1)?;
            Ok((file_id, relative_path))
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .filter(|(_, relative_path)| relative_path.ends_with("_mihome.sqlite"))
        .collect();
    drop(mstmt);
    drop(manifest_conn);

    if file_ids.is_empty() {
        // `NOTFOUND:` prefix routes the frontend to the "no tokens" result view.
        return Err(
            "NOTFOUND:Mi Home database (*_mihome.sqlite) was not found in this backup. Make sure the \
             Xiaomi Home app is installed, devices are added, and you opened the app before \
             creating the backup."
                .to_string(),
        );
    }

    emit_progress(&app, 90, "Extracting device tokens…", "device tokens");

    let mut devices = Vec::new();

    for (file_id, relative_path) in file_ids {
        let entry = backup
            .get_file(&file_id)
            .map_err(|e| format!("Couldn't read metadata for {relative_path}: {e}"))?;

        let data = backup
            .decrypt_entry(&entry)
            .map_err(|e| e.to_string())?;

        let tmp = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
        std::fs::write(tmp.path(), &data).map_err(|e| e.to_string())?;

        let conn = rusqlite::Connection::open(tmp.path()).map_err(|e| e.to_string())?;
        let mut stmt = match conn.prepare("SELECT ZNAME, ZMODEL, ZLOCALIP, ZMAC, ZTOKEN FROM ZDEVICE")
        {
            Ok(s) => s,
            Err(_) => continue,
        };

        let rows = stmt
            .query_map([], |row| {
                let name: Option<String> = row.get(0)?;
                let model: Option<String> = row.get(1)?;
                let ip: Option<String> = row.get(2)?;
                let mac: Option<String> = row.get(3)?;
                let token: Option<String> = row.get(4)?;
                Ok((name, model, ip, mac, token))
            })
            .map_err(|e| e.to_string())?;

        for row in rows.flatten() {
            let (name, model, ip, mac, token) = row;
            let token = token
                .filter(|t| !t.is_empty())
                .map(|t| decrypt_ztoken(&t))
                .unwrap_or_default();

            devices.push(XiaomiDevice {
                name: name.unwrap_or_default(),
                model: model.unwrap_or_default(),
                ip: ip.unwrap_or_default(),
                mac: mac.unwrap_or_default(),
                token,
            });
        }
    }

    emit_progress(&app, 100, "Done", "device tokens");

    Ok(devices)
}

/// Writes the given tokens as pretty-printed JSON to `path` (chosen via the
/// frontend save dialog).
#[tauri::command]
fn export_tokens(tokens: serde_json::Value, path: String) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&tokens).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_access,
            request_access,
            list_backups,
            extract_tokens,
            export_tokens
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
