use std::fs;
use std::io::{self, Read};
use std::path::Path;

fn get_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(30))
        .build()
}

pub(crate) fn fetch_url_string(url: &str) -> Result<String, String> {
    let response = get_agent()
        .get(url)
        .call()
        .map_err(|e| format!("HTTP request failed for {url}: {e}"))?;

    if response.status() != 200 {
        return Err(format!(
            "HTTP request for {url} returned status {}",
            response.status()
        ));
    }

    let mut body = String::new();
    response
        .into_reader()
        .read_to_string(&mut body)
        .map_err(|e| format!("Failed to read response body for {url}: {e}"))?;

    Ok(body.trim().to_string())
}

pub(crate) fn download_url_to_file(url: &str, dest: &Path) -> Result<(), String> {
    let response = get_agent()
        .get(url)
        .call()
        .map_err(|e| format!("HTTP request failed for {url}: {e}"))?;

    if response.status() != 200 {
        return Err(format!(
            "HTTP request for {url} returned status {}",
            response.status()
        ));
    }

    let parent = dest
        .parent()
        .ok_or_else(|| format!("Invalid destination path for {url}: {}", dest.display()))?;

    fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create parent directory for {url}: {e}"))?;

    let mut temp_file = tempfile::NamedTempFile::new_in(parent).map_err(|e| {
        format!(
            "Failed to create temporary file in {} for {url}: {e}",
            parent.display()
        )
    })?;

    let mut reader = response.into_reader();
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read response stream from {url}: {e}"))?;
        if bytes_read == 0 {
            break;
        }
        io::Write::write_all(temp_file.as_file_mut(), &buffer[..bytes_read]).map_err(|e| {
            format!(
                "Failed to write to temporary file {} for {url}: {e}",
                temp_file.path().display()
            )
        })?;
    }

    temp_file
        .as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync file for {url}: {e}"))?;

    temp_file.persist(dest).map_err(|e| {
        format!(
            "Failed to rename temporary file to {} for {url}: {e}",
            dest.display()
        )
    })?;

    Ok(())
}
