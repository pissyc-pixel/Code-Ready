use std::{
    collections::VecDeque,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde::Serialize;

const LOG_PREVIEW_LIMIT: usize = 5000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPreview {
    pub lines: Vec<String>,
    pub total_lines: usize,
    pub truncated: bool,
    pub log_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLogsResult {
    pub zip_path: String,
    pub file_count: usize,
}

#[tauri::command]
pub async fn get_log_preview(limit: Option<usize>) -> Result<LogPreview, String> {
    log_preview_at_path(&logs_dir()?, limit.unwrap_or(LOG_PREVIEW_LIMIT))
}

#[tauri::command]
pub async fn export_logs() -> Result<ExportLogsResult, String> {
    export_logs_at_path(&logs_dir()?, &export_dir()?)
}

#[tauri::command]
pub async fn open_full_log_file() -> Result<(), String> {
    let log = latest_log_file(&logs_dir()?)?.ok_or_else(|| "No log file is available".to_string())?;
    tauri_plugin_opener::open_path(log.to_string_lossy().to_string(), None::<&str>)
        .map_err(|error| format!("failed to open log file: {error}"))
}

#[tauri::command]
pub async fn open_log_directory() -> Result<(), String> {
    let dir = logs_dir()?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    tauri_plugin_opener::open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|error| format!("failed to open log directory: {error}"))
}

fn logs_dir() -> Result<PathBuf, String> {
    appdata_dir().map(|appdata| appdata.join("logs"))
}

fn export_dir() -> Result<PathBuf, String> {
    appdata_dir().map(|appdata| appdata.join("exports"))
}

fn appdata_dir() -> Result<PathBuf, String> {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA is not available".to_string())?;
    Ok(appdata.join("ai-coding-installer"))
}

fn log_preview_at_path(log_dir: &Path, max_lines: usize) -> Result<LogPreview, String> {
    let entries = collect_log_files(log_dir)?;
    let mut lines = VecDeque::with_capacity(max_lines.min(1));
    let mut total_lines = 0usize;

    for entry in entries {
        let name = archive_file_name(&entry);
        let content = fs::read_to_string(&entry).unwrap_or_default();
        for line in content.lines() {
            total_lines += 1;
            if max_lines > 0 && lines.len() == max_lines {
                lines.pop_front();
            }
            if max_lines > 0 {
                lines.push_back(format!("[{name}] {line}"));
            }
        }
    }

    Ok(LogPreview {
        lines: lines.into_iter().collect(),
        total_lines,
        truncated: total_lines > max_lines,
        log_dir: log_dir.to_string_lossy().to_string(),
    })
}

fn export_logs_at_path(log_dir: &Path, export_dir: &Path) -> Result<ExportLogsResult, String> {
    fs::create_dir_all(log_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(export_dir).map_err(|error| error.to_string())?;

    let entries = collect_log_files(log_dir)?;
    let zip_path = export_dir.join(format!(
        "diagnostics-{}.zip",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    ));

    let mut zip_entries = Vec::new();
    let mut summary = String::from(
        "Code-ready diagnostics export\nLogs are written by the Rust backend after line-by-line redaction.\n\n",
    );
    for entry in &entries {
        let name = archive_file_name(entry);
        let bytes = fs::read(entry).map_err(|error| error.to_string())?;
        summary.push_str(&format!("logs/{name}\t{} bytes\n", bytes.len()));
        zip_entries.push((format!("logs/{name}"), bytes));
    }
    zip_entries.push(("summary.txt".to_string(), summary.into_bytes()));

    write_store_zip(&zip_path, &zip_entries)?;

    Ok(ExportLogsResult {
        zip_path: zip_path.to_string_lossy().to_string(),
        file_count: entries.len(),
    })
}

fn latest_log_file(log_dir: &Path) -> Result<Option<PathBuf>, String> {
    let mut entries = collect_log_files(log_dir)?;
    entries.sort_by_key(|path| fs::metadata(path).and_then(|metadata| metadata.modified()).ok());
    Ok(entries.pop())
}

fn collect_log_files(log_dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !log_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(log_dir).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_file() && path.extension().and_then(|value| value.to_str()) == Some("log") {
            entries.push(path);
        }
    }
    entries.sort();
    Ok(entries)
}

fn archive_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown.log")
        .replace(['\\', '/'], "_")
}

fn write_store_zip(path: &Path, entries: &[(String, Vec<u8>)]) -> Result<(), String> {
    let mut file = File::create(path).map_err(|error| error.to_string())?;
    let mut central_directory = Vec::new();
    let mut offset = 0u32;

    for (name, data) in entries {
        let name_bytes = name.as_bytes();
        let crc = crc32(data);
        let size = data.len() as u32;
        let name_len = name_bytes.len() as u16;
        let local_offset = offset;

        write_u32(&mut file, 0x0403_4b50)?;
        write_u16(&mut file, 20)?;
        write_u16(&mut file, 0)?;
        write_u16(&mut file, 0)?;
        write_u16(&mut file, 0)?;
        write_u16(&mut file, 0)?;
        write_u32(&mut file, crc)?;
        write_u32(&mut file, size)?;
        write_u32(&mut file, size)?;
        write_u16(&mut file, name_len)?;
        write_u16(&mut file, 0)?;
        file.write_all(name_bytes)
            .map_err(|error| error.to_string())?;
        file.write_all(data).map_err(|error| error.to_string())?;

        offset += 30 + name_bytes.len() as u32 + size;

        write_u32(&mut central_directory, 0x0201_4b50)?;
        write_u16(&mut central_directory, 20)?;
        write_u16(&mut central_directory, 20)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u32(&mut central_directory, crc)?;
        write_u32(&mut central_directory, size)?;
        write_u32(&mut central_directory, size)?;
        write_u16(&mut central_directory, name_len)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u16(&mut central_directory, 0)?;
        write_u32(&mut central_directory, 0)?;
        write_u32(&mut central_directory, local_offset)?;
        central_directory
            .write_all(name_bytes)
            .map_err(|error| error.to_string())?;
    }

    let central_offset = offset;
    file.write_all(&central_directory)
        .map_err(|error| error.to_string())?;
    write_u32(&mut file, 0x0605_4b50)?;
    write_u16(&mut file, 0)?;
    write_u16(&mut file, 0)?;
    write_u16(&mut file, entries.len() as u16)?;
    write_u16(&mut file, entries.len() as u16)?;
    write_u32(&mut file, central_directory.len() as u32)?;
    write_u32(&mut file, central_offset)?;
    write_u16(&mut file, 0)?;
    Ok(())
}

fn write_u16<W: Write>(writer: &mut W, value: u16) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| error.to_string())
}

fn write_u32<W: Write>(writer: &mut W, value: u32) -> Result<(), String> {
    writer
        .write_all(&value.to_le_bytes())
        .map_err(|error| error.to_string())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::{export_logs_at_path, log_preview_at_path};
    use std::{fs, path::PathBuf};

    #[test]
    fn log_preview_keeps_only_the_latest_5000_lines() {
        let dir = unique_test_dir("preview");
        fs::create_dir_all(&dir).expect("create log dir");
        let path = dir.join("install.log");
        let content = (0..5002)
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, content).expect("write log");

        let preview = log_preview_at_path(&dir, 5000).expect("read preview");

        assert_eq!(preview.total_lines, 5002);
        assert!(preview.truncated);
        assert_eq!(preview.lines.len(), 5000);
        assert!(preview.lines[0].contains("line-2"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn export_logs_writes_zip_with_summary_without_uploading() {
        let dir = unique_test_dir("logs");
        let export_dir = unique_test_dir("exports");
        fs::create_dir_all(&dir).expect("create log dir");
        fs::write(dir.join("git.log"), "[stdout] token=[REDACTED]\n").expect("write log");

        let result = export_logs_at_path(&dir, &export_dir).expect("export logs");
        let bytes = fs::read(&result.zip_path).expect("read zip");

        assert_eq!(result.file_count, 1);
        assert!(bytes.starts_with(&[0x50, 0x4b, 0x03, 0x04]));
        assert!(bytes.windows("logs/git.log".len()).any(|item| item == b"logs/git.log"));
        assert!(bytes.windows("summary.txt".len()).any(|item| item == b"summary.txt"));

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(export_dir);
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ai-coding-logs-test-{name}-{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }
}
