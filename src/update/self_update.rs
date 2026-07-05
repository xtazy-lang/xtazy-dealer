use std::fs;

use crate::error::{DealerError, DealerResult};
use crate::state::DealerState;
use crate::support::archive::extract_archive;
use crate::support::checksum::compute_file_sha256;
use crate::support::host::{detect_host_target, replace_current_binary};
use crate::support::net::{download_url_to_file, fetch_url_string};
use crate::support::signing::verify_xsig_signature;

pub(crate) fn run_self_update(state: &DealerState) -> DealerResult<String> {
    if std::env::consts::OS == "windows" {
        return Err(DealerError::Backend(
            "automatic self-update is not supported on Windows".to_string(),
        ));
    }

    let version_txt_url = crate::constants::web::VERSION_TXT_URL;
    let latest_version = fetch_url_string(version_txt_url)
        .map_err(|e| DealerError::Backend(format!("failed to fetch latest dealer version: {e}")))?;

    let current_version = env!("CARGO_PKG_VERSION");
    if latest_version == current_version {
        return Ok(format!("dealer is already up to date ({current_version})"));
    }

    let temp_dir = state.cache_dir().join("temp_self_update");
    fs::create_dir_all(&temp_dir).map_err(|e| DealerError::io(&temp_dir, e))?;

    let tsv_file = temp_dir.join(crate::constants::files::TARGETS_TSV);
    let xsig_file = temp_dir.join(crate::constants::files::TARGETS_TSV_XSIGFILE);

    download_url_to_file(crate::constants::web::TARGETS_TSV_URL, &tsv_file)
        .map_err(|e| DealerError::Backend(format!("failed to download targets.tsv: {e}")))?;
    download_url_to_file(crate::constants::web::TARGETS_TSV_XSIGFILE_URL, &xsig_file).map_err(
        |e| DealerError::Backend(format!("failed to download targets.tsv.xsigfile: {e}")),
    )?;

    let master_public_bytes = include_bytes!("../trust/master.public");
    verify_xsig_signature(master_public_bytes, &tsv_file, &xsig_file).map_err(|e| {
        DealerError::Backend(format!("targets.tsv signature verification failed: {e}"))
    })?;

    let (os, arch) = detect_host_target()
        .map_err(|e| DealerError::Backend(format!("failed to detect host target: {e}")))?;
    let content = fs::read_to_string(&tsv_file).map_err(|e| DealerError::io(&tsv_file, e))?;
    let (suffix, sha256) = parse_targets_tsv(&content, &os, &arch)
        .map_err(|e| DealerError::Backend(format!("failed to parse targets.tsv: {e}")))?;

    let archive_url = crate::constants::web::dealer_archive_url(&latest_version, &suffix);
    let archive_file = temp_dir.join("dealer_archive.tar.gz");
    download_url_to_file(&archive_url, &archive_file)
        .map_err(|e| DealerError::Backend(format!("failed to download dealer archive: {e}")))?;

    let actual_sha = compute_file_sha256(&archive_file).map_err(|e| {
        DealerError::Backend(format!("failed to compute hash of downloaded archive: {e}"))
    })?;
    if actual_sha != sha256 {
        fs::remove_dir_all(&temp_dir).ok();
        return Err(DealerError::Backend(format!(
            "checksum mismatch for dealer archive: expected {sha256}, got {actual_sha}"
        )));
    }

    let extract_dir = temp_dir.join("extract");
    extract_archive(&archive_file, &extract_dir)
        .map_err(|e| DealerError::Backend(format!("failed to extract dealer archive: {e}")))?;

    let new_binary = extract_dir
        .join(crate::constants::executables::EXE_DEALER)
        .join(format!(
            "{}{}",
            crate::constants::executables::EXE_DEALER,
            std::env::consts::EXE_SUFFIX
        ));
    if !new_binary.is_file() {
        fs::remove_dir_all(&temp_dir).ok();
        return Err(DealerError::Backend(
            "dealer binary was not found in extracted archive".to_string(),
        ));
    }

    replace_current_binary(&new_binary).map_err(|e| {
        DealerError::Backend(format!("failed to replace current dealer binary: {e}"))
    })?;

    fs::remove_dir_all(&temp_dir).ok();

    Ok(format!(
        "Updated dealer: {current_version} -> {latest_version}"
    ))
}

pub(crate) fn parse_targets_tsv(
    content: &str,
    os: &str,
    arch: &str,
) -> Result<(String, String), String> {
    let key = format!("{os}_{arch}");
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 && parts[1] == key {
            return Ok((parts[2].to_string(), parts[3].to_string()));
        }
    }
    Err(format!("no matching row found in targets.tsv for {key}"))
}
