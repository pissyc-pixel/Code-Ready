use std::{path::PathBuf, time::Duration};

use crate::config::{AppConfig, CcSwitchDownloadSource, CcSwitchDownloadSourceKind};

use super::runner::InstallCommandSpec;

const DEFAULT_MIN_FILE_SIZE_BYTES: u64 = 64 * 1024;

pub fn command_spec(config: &AppConfig) -> Result<InstallCommandSpec, String> {
    let sources = ordered_enabled_sources(&config.ccswitch_download_sources);
    if sources.is_empty() {
        return Err(
            "No configured ccSwitch download sources are available. Please save a manual ccSwitch path."
                .to_string(),
        );
    }

    let target_dir = managed_ccswitch_dir()?;
    let script = build_download_script(&sources, &target_dir);

    Ok(InstallCommandSpec {
        program: "powershell.exe".to_string(),
        args: vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-Command".to_string(),
            script,
        ],
        envs: Vec::new(),
        timeout: Duration::from_secs(30 * 60),
        started_suggestion: Some(
            "Trying configured ccSwitch download sources in ascending priority order."
                .to_string(),
        ),
        success_suggestion: Some(
            "ccSwitch download completed. Re-checking the managed path and prompting manual path fallback if needed."
                .to_string(),
        ),
        failure_suggestion: Some(
            "All configured ccSwitch sources failed. Please save a manual ccSwitch path."
                .to_string(),
        ),
        detect_after: vec!["ccswitch".to_string()],
    })
}

pub fn ordered_enabled_sources(sources: &[CcSwitchDownloadSource]) -> Vec<CcSwitchDownloadSource> {
    let mut ordered = sources
        .iter()
        .filter(|source| source.enabled && !source.url.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    ordered.sort_by_key(|source| source.priority);
    ordered
}

fn build_download_script(sources: &[CcSwitchDownloadSource], target_dir: &PathBuf) -> String {
    let final_path = target_dir.join("ccswitch.exe");
    let source_entries = sources
        .iter()
        .map(build_source_entry)
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "$ErrorActionPreference = 'Stop'; \
$targetDir = '{target_dir}'; \
$finalPath = '{final_path}'; \
New-Item -ItemType Directory -Force -Path $targetDir | Out-Null; \
$sources = @({source_entries}); \
foreach ($source in $sources) {{ \
  Write-Output (\"Trying source [{{0}}] priority={{1}}\" -f $source.Name, $source.Priority); \
  $downloadPath = Join-Path $targetDir (\"ccswitch-\" + $source.Priority + (if ($source.Kind -eq 'direct_zip') {{ '.zip' }} else {{ '.exe' }})); \
  try {{ \
    Invoke-WebRequest -Uri $source.Url -OutFile $downloadPath -UseBasicParsing; \
    if (-not (Test-Path $downloadPath)) {{ throw 'downloaded file was not created'; }} \
    $size = (Get-Item $downloadPath).Length; \
    if ($size -lt [int64]$source.MinFileSizeBytes) {{ throw (\"downloaded file too small: {{0}} bytes\" -f $size); }} \
    if ($source.Sha256) {{ \
      $actualHash = (Get-FileHash -Algorithm SHA256 -Path $downloadPath).Hash.ToLowerInvariant(); \
      if ($actualHash -ne $source.Sha256.ToLowerInvariant()) {{ throw (\"sha256 mismatch: {{0}}\" -f $actualHash); }} \
    }} \
    if ($source.Kind -eq 'direct_zip') {{ throw 'direct_zip sources are not supported in V0.4; extract manually and save ccswitchPath'; }} \
    Copy-Item -Force $downloadPath $finalPath; \
    Write-Output (\"Source [{{0}}] succeeded: {{1}}\" -f $source.Name, $finalPath); \
    exit 0; \
  }} catch {{ \
    Write-Error (\"Source [{{0}}] failed: {{1}}\" -f $source.Name, $_.Exception.Message); \
  }} \
}}; \
throw 'All configured ccSwitch sources failed. Please save a manual ccSwitch path.'",
        target_dir = ps_literal(&target_dir.display().to_string()),
        final_path = ps_literal(&final_path.display().to_string()),
        source_entries = source_entries
    )
}

fn build_source_entry(source: &CcSwitchDownloadSource) -> String {
    let min_file_size_bytes = source
        .min_file_size_bytes
        .unwrap_or(DEFAULT_MIN_FILE_SIZE_BYTES);
    let sha256 = source.sha256.clone().unwrap_or_default();
    let kind = match source.kind {
        CcSwitchDownloadSourceKind::DirectExe => "direct_exe",
        CcSwitchDownloadSourceKind::DirectZip => "direct_zip",
    };

    format!(
        "@{{ Name='{name}'; Url='{url}'; Priority={priority}; Kind='{kind}'; Sha256='{sha256}'; MinFileSizeBytes={min_file_size_bytes} }}",
        name = ps_literal(&source.name),
        url = ps_literal(&source.url),
        priority = source.priority,
        kind = kind,
        sha256 = ps_literal(&sha256),
        min_file_size_bytes = min_file_size_bytes
    )
}

fn ps_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn managed_ccswitch_dir() -> Result<PathBuf, String> {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA is not available".to_string())?;
    Ok(appdata.join("ai-coding-installer").join("ccswitch"))
}

#[cfg(test)]
mod tests {
    use super::{build_download_script, command_spec, ordered_enabled_sources};
    use crate::config::{AppConfig, CcSwitchDownloadSource, CcSwitchDownloadSourceKind};
    use std::path::PathBuf;

    fn source(name: &str, priority: i32, enabled: bool, url: &str) -> CcSwitchDownloadSource {
        CcSwitchDownloadSource {
            name: name.to_string(),
            url: url.to_string(),
            priority,
            enabled,
            kind: CcSwitchDownloadSourceKind::DirectExe,
            sha256: None,
            min_file_size_bytes: Some(1024),
        }
    }

    #[test]
    fn orders_enabled_sources_by_priority() {
        let ordered = ordered_enabled_sources(&[
            source("third", 30, true, "https://example.invalid/3.exe"),
            source("disabled", 5, false, "https://example.invalid/disabled.exe"),
            source("second", 20, true, "https://example.invalid/2.exe"),
            source("first", 10, true, "https://example.invalid/1.exe"),
            source("blank", 1, true, " "),
        ]);

        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].name, "first");
        assert_eq!(ordered[1].name, "second");
        assert_eq!(ordered[2].name, "third");
    }

    #[test]
    fn command_spec_requires_at_least_one_enabled_source() {
        let error = command_spec(&AppConfig::default()).expect_err("empty source list should fail");
        assert!(error.contains("manual ccSwitch path"));
    }

    #[test]
    fn generated_script_keeps_priority_order() {
        let ordered = ordered_enabled_sources(&[
            source("b", 20, true, "https://example.invalid/b.exe"),
            source("a", 10, true, "https://example.invalid/a.exe"),
        ]);
        let script = build_download_script(&ordered, &PathBuf::from("C:\\temp\\ccswitch"));

        let first_index = script.find("https://example.invalid/a.exe").expect("first source");
        let second_index = script.find("https://example.invalid/b.exe").expect("second source");
        assert!(first_index < second_index);
        assert!(script.contains("Please save a manual ccSwitch path."));
    }
}
