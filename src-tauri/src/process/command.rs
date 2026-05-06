use std::{ffi::OsStr, process::Command};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn new_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    apply_no_window(&mut command);
    command
}

pub fn apply_no_window(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        command.creation_flags(windows_creation_flags());
    }

    command
}

#[cfg(windows)]
pub fn windows_creation_flags() -> u32 {
    CREATE_NO_WINDOW
}

#[cfg(not(windows))]
pub fn windows_creation_flags() -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::windows_creation_flags;

    #[test]
    fn uses_create_no_window_flag_on_windows() {
        #[cfg(windows)]
        assert_eq!(windows_creation_flags(), 0x0800_0000);
    }

    #[test]
    fn uses_zero_flag_on_non_windows() {
        #[cfg(not(windows))]
        assert_eq!(windows_creation_flags(), 0);
    }
}
