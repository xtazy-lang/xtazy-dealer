use crate::error::{DealerError, DealerResult};
use crate::state::DealerState;
use crate::support::archive::extract_archive;
use crate::support::checksum::compute_file_sha256;
use crate::support::net::{download_url_to_file, fetch_url_string};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

thread_local! {
    static MOCK_VERSIONS: RefCell<Option<String>> = const { RefCell::new(None) };
    static MOCK_METADATA: RefCell<Option<String>> = const { RefCell::new(None) };
    static MOCK_VERSIONS_MAP: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static MOCK_METADATA_MAP: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[cfg(test)]
pub(crate) fn set_mock_registry(versions: Option<String>, metadata: Option<String>) {
    MOCK_VERSIONS.with(|v| *v.borrow_mut() = versions);
    MOCK_METADATA.with(|m| *m.borrow_mut() = metadata);
}

#[cfg(test)]
pub(crate) fn set_mock_registry_map(
    versions: HashMap<String, String>,
    metadata: HashMap<String, String>,
) {
    MOCK_VERSIONS_MAP.with(|v| *v.borrow_mut() = versions);
    MOCK_METADATA_MAP.with(|m| *m.borrow_mut() = metadata);
}

#[derive(Debug, Clone)]
pub(crate) struct RegistryVersionEntry {
    pub(crate) version: String,
    pub(crate) _sha256: String,
    pub(crate) _url: String,
}

pub(crate) fn fetch_registry_versions(name: &str) -> DealerResult<Vec<RegistryVersionEntry>> {
    let map_val = MOCK_VERSIONS_MAP.with(|m| m.borrow().get(name).cloned());
    let content = if let Some(mocked) = map_val {
        mocked
    } else if let Some(mocked) = MOCK_VERSIONS.with(|v| v.borrow().clone()) {
        mocked
    } else {
        let url = crate::constants::web::package_versions_url(name);
        fetch_url_string(&url).map_err(|e| {
            DealerError::Package(format!("failed to fetch package versions for {name}: {e}"))
        })?
    };

    let mut entries = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(DealerError::Package(format!(
                "invalid versions list format for {name}: '{line}'"
            )));
        }
        let version = parts[0].to_string();
        if !parts[1].starts_with(crate::constants::components::SHA256_PREFIX) {
            return Err(DealerError::Package(format!(
                "invalid hash prefix in versions list for {name}: '{}'",
                parts[1]
            )));
        }
        let sha256 = parts[1][crate::constants::components::SHA256_PREFIX.len()..].to_string();
        let download_url = parts[2].to_string();
        entries.push(RegistryVersionEntry {
            version,
            _sha256: sha256,
            _url: download_url,
        });
    }
    Ok(entries)
}

pub(crate) fn resolve_registry_dependency(
    state: &DealerState,
    name: &str,
    version: &str,
) -> DealerResult<PathBuf> {
    let pkg_dir = state
        .cache_dir()
        .join(crate::constants::dirs::PACKAGES_DIR)
        .join(name)
        .join(version);
    let source_dir = pkg_dir.join(crate::constants::dirs::SOURCE_DIR);
    if source_dir.is_dir() {
        return Ok(source_dir);
    }

    // Lock cache write
    let lock_file = pkg_dir.join("lock");
    fs::create_dir_all(&pkg_dir).map_err(|e| DealerError::io(&pkg_dir, e))?;
    let f = fs::File::create(&lock_file).map_err(|e| DealerError::io(&lock_file, e))?;
    let mut lock = fd_lock::RwLock::new(f);
    let _write_lock = lock
        .write()
        .map_err(|e| DealerError::Package(format!("failed to lock package cache: {e}")))?;

    if source_dir.is_dir() {
        return Ok(source_dir);
    }

    // Fetch version metadata
    let key = format!("{name}-{version}");
    let map_val = MOCK_METADATA_MAP.with(|m| m.borrow().get(&key).cloned());
    let meta_str = if let Some(mocked) = map_val {
        mocked
    } else if let Some(mocked) = MOCK_METADATA.with(|m| m.borrow().clone()) {
        mocked
    } else {
        let url = crate::constants::web::package_version_url(name, version);
        fetch_url_string(&url)
            .map_err(|e| DealerError::Package(format!("failed to fetch package metadata: {e}")))?
    };
    let (tar_url, expected_sha) = parse_registry_metadata(&meta_str, version)?;

    let archive_path = pkg_dir.join(format!(
        "{}-{}{}",
        name,
        version,
        crate::constants::files::TAR_GZ_SUFFIX
    ));
    download_url_to_file(&tar_url, &archive_path)
        .map_err(|e| DealerError::Package(format!("failed to download registry archive: {e}")))?;

    let computed_sha = compute_file_sha256(&archive_path).map_err(|e| {
        DealerError::Package(format!("failed to compute hash of downloaded archive: {e}"))
    })?;
    if computed_sha != expected_sha {
        return Err(DealerError::Package(format!(
            "Checksum verification failed for registry package {name} {version}: expected {expected_sha}, computed {computed_sha}"
        )));
    }

    let temp_source_dir = pkg_dir.join("temp_source");
    extract_archive(&archive_path, &temp_source_dir)
        .map_err(|e| DealerError::Package(format!("failed to extract archive: {e}")))?;

    fs::rename(&temp_source_dir, &source_dir).map_err(|e| DealerError::io(&source_dir, e))?;
    Ok(source_dir)
}

pub(crate) fn parse_registry_metadata(
    meta_str: &str,
    expected_version: &str,
) -> DealerResult<(String, String)> {
    let parts: Vec<&str> = meta_str.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(DealerError::Package(format!(
            "invalid package metadata format: '{meta_str}'"
        )));
    }
    let returned_version = parts[0];
    if returned_version != expected_version {
        return Err(DealerError::Package(format!(
            "version mismatch: registry returned '{returned_version}' but expected '{expected_version}'"
        )));
    }
    if !parts[1].starts_with(crate::constants::components::SHA256_PREFIX) {
        return Err(DealerError::Package(format!(
            "registry metadata hash must start with '{}', got '{}'",
            crate::constants::components::SHA256_PREFIX,
            parts[1]
        )));
    }
    let expected_sha = parts[1][crate::constants::components::SHA256_PREFIX.len()..].to_string();
    let tar_url = parts[2].to_string();
    Ok((tar_url, expected_sha))
}
