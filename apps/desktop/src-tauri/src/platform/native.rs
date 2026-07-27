use std::env;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::domain::contracts::{PlatformId, ToolId};

use super::{
    CandidateKind, CandidateOrigin, ExecutableCandidate, PlatformAdapter, PlatformProbeError,
};

#[derive(Debug)]
pub struct UnsupportedPlatformError;

impl Display for UnsupportedPlatformError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("unsupported Code-Ready platform")
    }
}

impl std::error::Error for UnsupportedPlatformError {}

#[derive(Debug)]
pub struct NativePlatformAdapter {
    platform: PlatformId,
}

impl NativePlatformAdapter {
    pub fn new() -> Result<Self, UnsupportedPlatformError> {
        Self::current()
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    fn current() -> Result<Self, UnsupportedPlatformError> {
        Ok(Self {
            platform: PlatformId::WindowsX64,
        })
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn current() -> Result<Self, UnsupportedPlatformError> {
        Ok(Self {
            platform: PlatformId::MacosArm64,
        })
    }

    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    fn current() -> Result<Self, UnsupportedPlatformError> {
        Err(UnsupportedPlatformError)
    }
}

impl PlatformAdapter for NativePlatformAdapter {
    fn platform(&self) -> PlatformId {
        self.platform.clone()
    }

    fn tool_candidates(
        &self,
        tool_id: ToolId,
    ) -> Result<Vec<ExecutableCandidate>, PlatformProbeError> {
        enumerate_candidates(self.platform.clone(), tool_id)
    }
}

pub fn known_location_templates(platform: PlatformId, tool_id: ToolId) -> Vec<&'static str> {
    match (platform, tool_id) {
        (PlatformId::WindowsX64, ToolId::Git) => vec![
            r"%ProgramFiles%\Git\cmd\git.exe",
            r"%ProgramFiles%\Git\bin\git.exe",
        ],
        (PlatformId::MacosArm64, ToolId::Git) => vec![
            "/opt/homebrew/bin/git",
            "/usr/local/bin/git",
            "/usr/bin/git",
        ],
        (PlatformId::WindowsX64, ToolId::ClaudeCode) => {
            vec![r"%USERPROFILE%\.local\bin\claude.exe"]
        }
        (PlatformId::MacosArm64, ToolId::ClaudeCode) => vec!["$HOME/.local/bin/claude"],
        (PlatformId::WindowsX64, ToolId::CodexCli) => {
            vec![r"%LOCALAPPDATA%\Programs\OpenAI\Codex\bin\codex.exe"]
        }
        (PlatformId::MacosArm64, ToolId::CodexCli) => vec!["$HOME/.local/bin/codex"],
        _ => vec![],
    }
}

fn enumerate_candidates(
    platform: PlatformId,
    tool_id: ToolId,
) -> Result<Vec<ExecutableCandidate>, PlatformProbeError> {
    let Some(base_name) = tool_base_name(&tool_id) else {
        return Ok(vec![]);
    };
    let path_value = env::var_os("PATH").ok_or(PlatformProbeError::EnvironmentUnavailable)?;
    let mut candidates = Vec::new();

    for directory in env::split_paths(&path_value) {
        for name in candidate_names(&platform, base_name) {
            add_existing_candidate(
                &mut candidates,
                directory.join(name),
                CandidateOrigin::Path,
                &platform,
            )?;
        }
    }

    for template in known_location_templates(platform.clone(), tool_id) {
        if let Some(path) = expand_template(template) {
            add_existing_candidate(
                &mut candidates,
                path,
                CandidateOrigin::KnownLocation,
                &platform,
            )?;
        }
    }

    Ok(candidates)
}

fn tool_base_name(tool_id: &ToolId) -> Option<&'static str> {
    match tool_id {
        ToolId::Git => Some("git"),
        ToolId::ClaudeCode => Some("claude"),
        ToolId::CodexCli => Some("codex"),
        ToolId::Winget | ToolId::Nodejs => None,
    }
}

fn candidate_names(platform: &PlatformId, base_name: &str) -> Vec<String> {
    match platform {
        PlatformId::WindowsX64 => vec![
            format!("{base_name}.exe"),
            format!("{base_name}.cmd"),
            format!("{base_name}.bat"),
            format!("{base_name}.ps1"),
        ],
        PlatformId::MacosArm64 => vec![base_name.to_owned()],
    }
}

fn add_existing_candidate(
    candidates: &mut Vec<ExecutableCandidate>,
    path: PathBuf,
    origin: CandidateOrigin,
    platform: &PlatformId,
) -> Result<(), PlatformProbeError> {
    if candidates.iter().any(|candidate| candidate.path == path) {
        return Ok(());
    }

    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(PlatformProbeError::PathUnreadable),
    };
    if !metadata.is_file() {
        return Ok(());
    }

    let kind = classify_candidate(&path, platform)?;
    candidates.push(ExecutableCandidate::new(
        path.clone(),
        display_path(&path, platform),
        origin,
        kind,
    ));
    Ok(())
}

fn classify_candidate(
    path: &Path,
    platform: &PlatformId,
) -> Result<CandidateKind, PlatformProbeError> {
    let magic = read_magic(path)?;
    Ok(classify_bytes(platform, path, &magic))
}

fn classify_bytes(platform: &PlatformId, path: &Path, magic: &[u8]) -> CandidateKind {
    if *platform == PlatformId::MacosArm64 && path == Path::new("/usr/bin/git") {
        return CandidateKind::AppleGitShim;
    }

    match platform {
        PlatformId::WindowsX64 => {
            if path.extension().and_then(|extension| extension.to_str()) == Some("exe")
                && magic.starts_with(b"MZ")
            {
                CandidateKind::NativeBinary
            } else {
                CandidateKind::NonNativeLauncher
            }
        }
        PlatformId::MacosArm64 => {
            if is_macho_magic(magic) {
                CandidateKind::NativeBinary
            } else {
                CandidateKind::NonNativeLauncher
            }
        }
    }
}

fn read_magic(path: &Path) -> Result<Vec<u8>, PlatformProbeError> {
    let mut file = File::open(path).map_err(|_| PlatformProbeError::PathUnreadable)?;
    let mut magic = [0_u8; 4];
    let bytes_read = file
        .read(&mut magic)
        .map_err(|_| PlatformProbeError::PathUnreadable)?;
    Ok(magic[..bytes_read].to_vec())
}

fn is_macho_magic(magic: &[u8]) -> bool {
    matches!(
        magic,
        [0xfe, 0xed, 0xfa, 0xce]
            | [0xce, 0xfa, 0xed, 0xfe]
            | [0xfe, 0xed, 0xfa, 0xcf]
            | [0xcf, 0xfa, 0xed, 0xfe]
            | [0xca, 0xfe, 0xba, 0xbe]
            | [0xbe, 0xba, 0xfe, 0xca]
    )
}

fn expand_template(template: &str) -> Option<PathBuf> {
    for (token, variable) in [
        ("$HOME", "HOME"),
        ("%USERPROFILE%", "USERPROFILE"),
        ("%LOCALAPPDATA%", "LOCALAPPDATA"),
        ("%ProgramFiles%", "ProgramFiles"),
    ] {
        if let Some(suffix) = template.strip_prefix(token) {
            let root = env::var_os(variable)?;
            return Some(PathBuf::from(root).join(suffix.trim_start_matches(['/', '\\'])));
        }
    }

    Some(PathBuf::from(template))
}

fn display_path(path: &Path, platform: &PlatformId) -> String {
    let home_variable = match platform {
        PlatformId::WindowsX64 => "USERPROFILE",
        PlatformId::MacosArm64 => "HOME",
    };
    let root = env::var_os(home_variable).map(PathBuf::from);
    display_path_with_root(path, platform, root.as_deref())
}

fn display_path_with_root(path: &Path, platform: &PlatformId, root: Option<&Path>) -> String {
    if let Some(root) = root {
        if *platform == PlatformId::WindowsX64 {
            if let Some(relative) = strip_windows_prefix(path, root) {
                return if relative.is_empty() {
                    "%USERPROFILE%".to_owned()
                } else {
                    format!("%USERPROFILE%{relative}")
                };
            }
        } else if let Ok(relative) = path.strip_prefix(root) {
            return if relative.as_os_str().is_empty() {
                "~".to_owned()
            } else {
                format!("~{}{}", std::path::MAIN_SEPARATOR, relative.display())
            };
        }
    }

    path.display().to_string()
}

fn strip_windows_prefix<'a>(path: &'a Path, root: &Path) -> Option<&'a str> {
    let path = path.to_str()?;
    let root = root.to_str()?.trim_end_matches(['\\', '/']);
    let path_lower = path.to_ascii_lowercase();
    let root_lower = root.to_ascii_lowercase();

    if path_lower == root_lower {
        return Some("");
    }
    if !path_lower.starts_with(&root_lower) {
        return None;
    }

    let root_length = root.len();
    let separator = path.as_bytes().get(root_length).copied()?;
    if separator != b'\\' && separator != b'/' {
        return None;
    }
    Some(&path[root_length..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{PlatformId, ToolId};

    #[test]
    fn executable_magic_classification_is_platform_specific_and_read_only() {
        assert_eq!(
            classify_bytes(
                &PlatformId::WindowsX64,
                Path::new("C:\\tools\\git.exe"),
                b"MZ\x90\0",
            ),
            CandidateKind::NativeBinary
        );
        assert_eq!(
            classify_bytes(
                &PlatformId::WindowsX64,
                Path::new("C:\\tools\\claude.cmd"),
                b"@echo off",
            ),
            CandidateKind::NonNativeLauncher
        );
        assert_eq!(
            classify_bytes(
                &PlatformId::MacosArm64,
                Path::new("/opt/homebrew/bin/codex"),
                &[0xcf, 0xfa, 0xed, 0xfe],
            ),
            CandidateKind::NativeBinary
        );
        assert_eq!(
            classify_bytes(
                &PlatformId::MacosArm64,
                Path::new("/usr/bin/git"),
                b"#!/bin/sh",
            ),
            CandidateKind::AppleGitShim
        );
    }

    #[test]
    fn native_known_locations_match_official_cli_installers() {
        assert_eq!(
            known_location_templates(PlatformId::MacosArm64, ToolId::ClaudeCode),
            vec!["$HOME/.local/bin/claude"]
        );
        assert_eq!(
            known_location_templates(PlatformId::WindowsX64, ToolId::ClaudeCode),
            vec!["%USERPROFILE%\\.local\\bin\\claude.exe"]
        );
        assert_eq!(
            known_location_templates(PlatformId::MacosArm64, ToolId::CodexCli),
            vec!["$HOME/.local/bin/codex"]
        );
        assert_eq!(
            known_location_templates(PlatformId::WindowsX64, ToolId::CodexCli),
            vec!["%LOCALAPPDATA%\\Programs\\OpenAI\\Codex\\bin\\codex.exe"]
        );
    }

    #[test]
    fn windows_home_redaction_ignores_path_case() {
        let displayed = display_path_with_root(
            Path::new(r"c:\users\alice\bin\git.exe"),
            &PlatformId::WindowsX64,
            Some(Path::new(r"C:\Users\Alice")),
        );

        assert_eq!(displayed, r"%USERPROFILE%\bin\git.exe");
    }
}
