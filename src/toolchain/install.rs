use std::fs;
use std::io;
use std::path::Path;

use crate::error::{DealerError, DealerResult};
use crate::state::{DealerState, XtazyParts};
use crate::support::archive::extract_archive;
use crate::support::checksum::compute_file_sha256;
use crate::support::host::detect_host_target;
use crate::support::net::download_url_to_file;
use crate::support::signing::verify_xsig_signature;

pub(crate) fn install_xtazy_composition(state: &DealerState, version: &str) -> DealerResult<()> {
    if std::env::consts::OS == "windows" {
        return Err(DealerError::Backend(
            "automatic xtazy toolchain installation is not supported on Windows".to_string(),
        ));
    }

    let lock_file = state.dealer_home.join("xtazy_update.lock");
    fs::create_dir_all(&state.dealer_home).map_err(|e| DealerError::io(&state.dealer_home, e))?;
    let f = fs::File::create(&lock_file).map_err(|e| DealerError::io(&lock_file, e))?;
    let mut lock = fd_lock::RwLock::new(f);
    let _write_lock = lock
        .write()
        .map_err(|e| DealerError::Backend(e.to_string()))?;

    let parts_url = crate::constants::web::xtazy_parts_url(version);
    let xsig_url = crate::constants::web::xtazy_parts_xsigfile_url(version);

    let temp_parts_dir = state.cache_dir().join("temp_xtazy_parts").join(version);
    fs::create_dir_all(&temp_parts_dir).map_err(|e| DealerError::io(&temp_parts_dir, e))?;

    let parts_file = temp_parts_dir.join(crate::constants::files::XTAZY_PARTS);
    let xsig_file = temp_parts_dir.join(crate::constants::files::XTAZY_PARTS_XSIGFILE);

    download_url_to_file(&parts_url, &parts_file)
        .map_err(|e| DealerError::Backend(format!("failed to download xtazy.parts: {e}")))?;
    download_url_to_file(&xsig_url, &xsig_file).map_err(|e| {
        DealerError::Backend(format!("failed to download xtazy.parts.xsigfile: {e}"))
    })?;

    let master_public_bytes = include_bytes!("../trust/master.public");
    verify_xsig_signature(master_public_bytes, &parts_file, &xsig_file).map_err(|e| {
        DealerError::Backend(format!(
            "signature verification failed for xtazy.parts: {e}"
        ))
    })?;

    let content = fs::read_to_string(&parts_file).map_err(|e| DealerError::io(&parts_file, e))?;
    let parsed = crate::state::parse_xtazy_parts(&content)
        .map_err(|e| DealerError::Backend(format!("failed to parse xtazy.parts: {e}")))?;

    install_piko_component(state, &parsed)?;
    install_rusttime_component(state, &parsed)?;
    install_std_component(state, &parsed)?;
    install_rust_backend_component(state, &parsed)?;

    let final_dir = state.toolchain_dir(version);
    fs::create_dir_all(&final_dir).map_err(|e| DealerError::io(&final_dir, e))?;

    fs::copy(
        &parts_file,
        final_dir.join(crate::constants::files::XTAZY_PARTS),
    )
    .map_err(|e| DealerError::io(final_dir.join(crate::constants::files::XTAZY_PARTS), e))?;
    fs::copy(
        &xsig_file,
        final_dir.join(crate::constants::files::XTAZY_PARTS_XSIGFILE),
    )
    .map_err(|e| {
        DealerError::io(
            final_dir.join(crate::constants::files::XTAZY_PARTS_XSIGFILE),
            e,
        )
    })?;

    fs::remove_dir_all(&temp_parts_dir).ok();

    Ok(())
}

fn install_piko_component(state: &DealerState, parsed: &XtazyParts) -> DealerResult<()> {
    let piko_ver = &parsed.piko_version;
    let target_dir = state
        .dealer_home
        .join(crate::constants::dirs::PIKO_COMPONENT_DIR)
        .join(piko_ver);
    let target_exe = target_dir.join(format!(
        "{}{}",
        crate::constants::executables::EXE_PIKO,
        std::env::consts::EXE_SUFFIX
    ));
    if target_exe.is_file() {
        return Ok(());
    }

    let temp_dir = state.cache_dir().join("temp_piko_install").join(piko_ver);
    fs::create_dir_all(&temp_dir).map_err(|e| DealerError::io(&temp_dir, e))?;

    let tsv_file = temp_dir.join(crate::constants::files::TARGETS_TSV);
    let xsig_file = temp_dir.join(crate::constants::files::TARGETS_TSV_XSIGFILE);

    download_url_to_file(
        &crate::constants::web::piko_targets_tsv_url(piko_ver),
        &tsv_file,
    )
    .map_err(|e| DealerError::Backend(format!("failed to download πko targets.tsv: {e}")))?;
    download_url_to_file(
        &crate::constants::web::piko_targets_tsv_xsigfile_url(piko_ver),
        &xsig_file,
    )
    .map_err(|e| {
        DealerError::Backend(format!("failed to download πko targets.tsv.xsigfile: {e}"))
    })?;

    let master_public_bytes = include_bytes!("../trust/master.public");
    verify_xsig_signature(master_public_bytes, &tsv_file, &xsig_file).map_err(|e| {
        DealerError::Backend(format!(
            "πko targets.tsv signature verification failed: {e}"
        ))
    })?;

    let tsv_sha = compute_file_sha256(&tsv_file).map_err(|e| {
        DealerError::Backend(format!("failed to compute hash of πko targets.tsv: {e}"))
    })?;
    let expected_tsv_sha = if parsed
        .piko_hash
        .starts_with(crate::constants::components::SHA256_PREFIX)
    {
        &parsed.piko_hash[crate::constants::components::SHA256_PREFIX.len()..]
    } else {
        &parsed.piko_hash
    };
    if tsv_sha != expected_tsv_sha {
        return Err(DealerError::Backend(format!(
            "πko targets.tsv checksum mismatch: expected {expected_tsv_sha}, got {tsv_sha}"
        )));
    }

    let (os, arch) = detect_host_target()
        .map_err(|e| DealerError::Backend(format!("failed to detect host target: {e}")))?;
    let tsv_content = fs::read_to_string(&tsv_file).map_err(|e| DealerError::io(&tsv_file, e))?;
    let (suffix, sha256) = crate::update::self_update::parse_targets_tsv(&tsv_content, &os, &arch)
        .map_err(|e| DealerError::Backend(format!("failed to parse πko targets.tsv: {e}")))?;

    let archive_url = crate::constants::web::piko_archive_url(piko_ver, &suffix);
    let archive_file = temp_dir.join("piko_archive.tar.gz");
    download_url_to_file(&archive_url, &archive_file)
        .map_err(|e| DealerError::Backend(format!("failed to download πko archive: {e}")))?;

    let actual_sha = compute_file_sha256(&archive_file).map_err(|e| {
        DealerError::Backend(format!(
            "failed to compute hash of downloaded πko archive: {e}"
        ))
    })?;
    if actual_sha != sha256 {
        return Err(DealerError::Backend(format!(
            "checksum mismatch for πko archive: expected {sha256}, got {actual_sha}"
        )));
    }

    let extract_dir = temp_dir.join("extract");
    extract_archive(&archive_file, &extract_dir)
        .map_err(|e| DealerError::Backend(format!("failed to extract πko archive: {e}")))?;

    let extracted_exe = extract_dir
        .join(crate::constants::executables::EXE_PIKO)
        .join(format!(
            "{}{}",
            crate::constants::executables::EXE_PIKO,
            std::env::consts::EXE_SUFFIX
        ));
    if !extracted_exe.is_file() {
        return Err(DealerError::Backend(
            "πko binary was not found in extracted archive".to_string(),
        ));
    }

    fs::create_dir_all(&target_dir).map_err(|e| DealerError::io(&target_dir, e))?;
    fs::rename(&extracted_exe, &target_exe).map_err(|e| DealerError::io(&target_exe, e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_exe)
            .map_err(|e| DealerError::io(&target_exe, e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_exe, perms).map_err(|e| DealerError::io(&target_exe, e))?;
    }

    fs::remove_dir_all(&temp_dir).ok();
    Ok(())
}

fn install_rusttime_component(state: &DealerState, parsed: &XtazyParts) -> DealerResult<()> {
    let ver = &parsed.rusttime_version;
    let target_dir = state
        .dealer_home
        .join(crate::constants::dirs::RUSTTIME_DIR)
        .join(ver);
    if target_dir.is_dir() {
        return Ok(());
    }

    let temp_dir = state.cache_dir().join("temp_rusttime_install").join(ver);
    fs::create_dir_all(&temp_dir).map_err(|e| DealerError::io(&temp_dir, e))?;

    let archive_url = crate::constants::web::rusttime_archive_url(ver);
    let archive_file = temp_dir.join("rusttime_archive.tar.gz");
    download_url_to_file(&archive_url, &archive_file)
        .map_err(|e| DealerError::Backend(format!("failed to download rusttime archive: {e}")))?;

    let actual_sha = compute_file_sha256(&archive_file).map_err(|e| {
        DealerError::Backend(format!("failed to compute hash of rusttime archive: {e}"))
    })?;
    let expected_sha = if parsed
        .rusttime_hash
        .starts_with(crate::constants::components::SHA256_PREFIX)
    {
        &parsed.rusttime_hash[crate::constants::components::SHA256_PREFIX.len()..]
    } else {
        &parsed.rusttime_hash
    };
    if actual_sha != expected_sha {
        return Err(DealerError::Backend(format!(
            "checksum mismatch for rusttime archive: expected {expected_sha}, got {actual_sha}"
        )));
    }

    let extract_dir = temp_dir.join("extract");
    extract_archive(&archive_file, &extract_dir)
        .map_err(|e| DealerError::Backend(format!("failed to extract rusttime archive: {e}")))?;

    fs::create_dir_all(target_dir.parent().unwrap())
        .map_err(|e| DealerError::io(target_dir.parent().unwrap(), e))?;
    fs::rename(&extract_dir, &target_dir).map_err(|e| DealerError::io(&target_dir, e))?;

    fs::remove_dir_all(&temp_dir).ok();
    Ok(())
}

fn install_std_component(state: &DealerState, parsed: &XtazyParts) -> DealerResult<()> {
    let ver = &parsed.std_version;
    let target_dir = state
        .dealer_home
        .join(crate::constants::dirs::STD_DIR)
        .join(ver);
    if target_dir.is_dir() {
        return Ok(());
    }

    let temp_dir = state.cache_dir().join("temp_std_install").join(ver);
    fs::create_dir_all(&temp_dir).map_err(|e| DealerError::io(&temp_dir, e))?;

    let archive_url = crate::constants::web::std_archive_url(ver);
    let archive_file = temp_dir.join("std_archive.tar.gz");
    download_url_to_file(&archive_url, &archive_file)
        .map_err(|e| DealerError::Backend(format!("failed to download std archive: {e}")))?;

    let actual_sha = compute_file_sha256(&archive_file)
        .map_err(|e| DealerError::Backend(format!("failed to compute hash of std archive: {e}")))?;
    let expected_sha = if parsed
        .std_hash
        .starts_with(crate::constants::components::SHA256_PREFIX)
    {
        &parsed.std_hash[crate::constants::components::SHA256_PREFIX.len()..]
    } else {
        &parsed.std_hash
    };
    if actual_sha != expected_sha {
        return Err(DealerError::Backend(format!(
            "checksum mismatch for std archive: expected {expected_sha}, got {actual_sha}"
        )));
    }

    let extract_dir = temp_dir.join("extract");
    extract_archive(&archive_file, &extract_dir)
        .map_err(|e| DealerError::Backend(format!("failed to extract std archive: {e}")))?;

    fs::create_dir_all(target_dir.parent().unwrap())
        .map_err(|e| DealerError::io(target_dir.parent().unwrap(), e))?;
    fs::rename(&extract_dir, &target_dir).map_err(|e| DealerError::io(&target_dir, e))?;

    fs::remove_dir_all(&temp_dir).ok();
    Ok(())
}

fn install_rust_backend_component(state: &DealerState, parsed: &XtazyParts) -> DealerResult<()> {
    let ver = &parsed.rust_version;
    let backend_id = format!("default-{ver}");
    let target_dir = state
        .dealer_home
        .join(crate::constants::dirs::RUST_DIR)
        .join(&backend_id);
    if target_dir
        .join(crate::constants::dirs::BIN_DIR)
        .join(format!(
            "{}{}",
            crate::constants::executables::EXE_CARGO,
            std::env::consts::EXE_SUFFIX
        ))
        .is_file()
    {
        return Ok(());
    }

    let temp_dir = state.cache_dir().join("temp_rust_install").join(ver);
    fs::create_dir_all(&temp_dir).map_err(|e| DealerError::io(&temp_dir, e))?;

    let manifest_url = crate::constants::web::rust_manifest_url(ver);
    let manifest_file = temp_dir.join("channel-rust.toml");
    download_url_to_file(&manifest_url, &manifest_file).map_err(|e| {
        DealerError::Backend(format!("failed to download Rust channel manifest: {e}"))
    })?;

    let actual_sha = compute_file_sha256(&manifest_file).map_err(|e| {
        DealerError::Backend(format!(
            "failed to compute hash of Rust channel manifest: {e}"
        ))
    })?;
    let expected_sha = if parsed
        .rust_hash
        .starts_with(crate::constants::components::SHA256_PREFIX)
    {
        &parsed.rust_hash[crate::constants::components::SHA256_PREFIX.len()..]
    } else {
        &parsed.rust_hash
    };
    if actual_sha != expected_sha {
        return Err(DealerError::Backend(format!(
            "checksum mismatch for Rust channel manifest: expected {expected_sha}, got {actual_sha}"
        )));
    }

    let (os, arch) = detect_host_target()
        .map_err(|e| DealerError::Backend(format!("failed to detect host target: {e}")))?;
    let target_triple = get_rust_target_triple(&os, &arch).map_err(DealerError::Backend)?;

    let manifest_content =
        fs::read_to_string(&manifest_file).map_err(|e| DealerError::io(&manifest_file, e))?;
    let mut download_url = None;
    let mut hash = None;
    let mut in_target = false;
    for line in manifest_content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("[pkg.rust.target.{}]", target_triple)) {
            in_target = true;
        } else if line.starts_with("[") {
            in_target = false;
        } else if in_target {
            if line.starts_with("xz_url =") {
                download_url = Some(
                    line.split("=")
                        .nth(1)
                        .unwrap()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.starts_with("xz_hash =") {
                hash = Some(
                    line.split("=")
                        .nth(1)
                        .unwrap()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.starts_with("url =") && download_url.is_none() {
                download_url = Some(
                    line.split("=")
                        .nth(1)
                        .unwrap()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.starts_with("hash =") && hash.is_none() {
                hash = Some(
                    line.split("=")
                        .nth(1)
                        .unwrap()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            }
        }
    }

    let download_url = download_url.ok_or_else(|| {
        DealerError::Backend(format!(
            "Rust package not available for target {target_triple}"
        ))
    })?;
    let hash = hash.ok_or_else(|| {
        DealerError::Backend(format!(
            "Rust package hash not found for target {target_triple}"
        ))
    })?;

    // Download official Rust archive
    let is_xz = download_url.ends_with(".xz");
    let archive_ext = if is_xz { ".tar.xz" } else { ".tar.gz" };
    let archive_file = temp_dir.join(format!("rust_archive{archive_ext}"));
    download_url_to_file(&download_url, &archive_file)
        .map_err(|e| DealerError::Backend(format!("failed to download Rust archive: {e}")))?;

    let actual_sha = compute_file_sha256(&archive_file).map_err(|e| {
        DealerError::Backend(format!(
            "failed to compute hash of downloaded Rust archive: {e}"
        ))
    })?;
    if actual_sha != hash {
        return Err(DealerError::Backend(format!(
            "checksum mismatch for Rust archive: expected {hash}, got {actual_sha}"
        )));
    }

    // Extract Rust archive
    let extract_dir = temp_dir.join("extract");
    extract_archive(&archive_file, &extract_dir)
        .map_err(|e| DealerError::Backend(format!("failed to extract Rust archive: {e}")))?;

    let mut top_dir = extract_dir.clone();
    let entries = fs::read_dir(&extract_dir)
        .map_err(|e| DealerError::io(&extract_dir, e))?
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    if entries.len() == 1 && entries[0].path().is_dir() {
        top_dir = entries[0].path();
    }

    let temp_backend_dir = temp_dir.join("temp_backend");
    fs::create_dir_all(temp_backend_dir.join("bin"))
        .map_err(|e| DealerError::io(temp_backend_dir.join("bin"), e))?;
    fs::create_dir_all(temp_backend_dir.join("lib"))
        .map_err(|e| DealerError::io(temp_backend_dir.join("lib"), e))?;

    // Copy rustc component binaries and libs
    let rustc_bin = top_dir.join("rustc").join("bin");
    if rustc_bin.is_dir() {
        copy_dir_all(&rustc_bin, &temp_backend_dir.join("bin"))
            .map_err(|e| DealerError::io(temp_backend_dir.join("bin"), e))?;
    }
    let rustc_lib = top_dir.join("rustc").join("lib");
    if rustc_lib.is_dir() {
        copy_dir_all(&rustc_lib, &temp_backend_dir.join("lib"))
            .map_err(|e| DealerError::io(temp_backend_dir.join("lib"), e))?;
    }

    // Copy cargo component binaries
    let cargo_bin = top_dir.join("cargo").join("bin");
    if cargo_bin.is_dir() {
        copy_dir_all(&cargo_bin, &temp_backend_dir.join("bin"))
            .map_err(|e| DealerError::io(temp_backend_dir.join("bin"), e))?;
    }

    // Copy rust-std component matching the host target triple
    let std_component_name = format!("rust-std-{}", target_triple);
    let std_component_dir = top_dir.join(&std_component_name);
    if !std_component_dir.is_dir() {
        return Err(DealerError::Backend(format!(
            "required rust-std component for target {} was not found in Rust package",
            target_triple
        )));
    }
    let std_lib_dir = std_component_dir.join("lib").join("rustlib");
    if !std_lib_dir.is_dir() {
        return Err(DealerError::Backend(format!(
            "invalid std component: missing lib/rustlib in {}",
            std_component_dir.display()
        )));
    }
    let dest_lib_dir = temp_backend_dir.join("lib").join("rustlib");
    copy_dir_all(&std_lib_dir, &dest_lib_dir).map_err(|e| DealerError::io(dest_lib_dir, e))?;

    // Move atomically to final target_dir
    fs::create_dir_all(target_dir.parent().unwrap())
        .map_err(|e| DealerError::io(target_dir.parent().unwrap(), e))?;
    fs::rename(&temp_backend_dir, &target_dir).map_err(|e| DealerError::io(&target_dir, e))?;

    fs::remove_dir_all(&temp_dir).ok();
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn get_rust_target_triple(os: &str, arch: &str) -> Result<String, String> {
    let os_lower = os.to_lowercase();
    let arch_lower = arch.to_lowercase();
    if os_lower.contains("darwin") {
        if arch_lower.contains("arm") || arch_lower.contains("aarch64") {
            Ok("aarch64-apple-darwin".to_string())
        } else {
            Ok("x86_64-apple-darwin".to_string())
        }
    } else if os_lower.contains("linux") {
        if arch_lower.contains("arm") || arch_lower.contains("aarch64") {
            Ok("aarch64-unknown-linux-gnu".to_string())
        } else {
            Ok("x86_64-unknown-linux-gnu".to_string())
        }
    } else {
        Err(format!("unsupported os for rust target mapping: {os}"))
    }
}
