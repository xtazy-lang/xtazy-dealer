use std::fs;
use std::path::Path;

pub(crate) fn detect_host_target() -> Result<(String, String), String> {
    let os = match std::env::consts::OS {
        "macos" => "Darwin".to_string(),
        "linux" => "Linux".to_string(),
        "windows" => "Windows".to_string(),
        other => return Err(format!("unsupported operating system: {other}")),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64".to_string(),
        "aarch64" => "arm64".to_string(),
        other => return Err(format!("unsupported architecture: {other}")),
    };
    Ok((os, arch))
}

pub(crate) fn replace_current_binary(new_binary_path: &Path) -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("failed to get current exe path: {e}"))?;
    let exe_dir = current_exe
        .parent()
        .ok_or_else(|| "no parent directory for current exe".to_string())?;

    let temp_exe = exe_dir.join("dealer.tmp");
    if temp_exe.exists() {
        fs::remove_file(&temp_exe).ok();
    }

    fs::copy(new_binary_path, &temp_exe).map_err(|e| format!("failed to copy new binary: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_exe)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_exe, perms).map_err(|e| e.to_string())?;
    }

    fs::rename(&temp_exe, &current_exe)
        .map_err(|e| format!("failed to replace current binary: {e}"))?;
    Ok(())
}

pub(crate) fn workspace_root() -> std::path::PathBuf {
    let compiled_workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtazy-dealer should live inside workspace root at build time")
        .to_path_buf();
    if compiled_workspace.join("xtazy-dealer").is_dir() {
        return compiled_workspace;
    }

    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}
