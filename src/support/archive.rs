use std::fs;
use std::path::Path;

fn is_path_safe(entry_path: &Path) -> bool {
    let mut depth = 0;
    for cmp in entry_path.components() {
        match cmp {
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                return false;
            }
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(_) => {
                depth += 1;
            }
        }
    }
    true
}

pub(crate) fn extract_archive(archive: &Path, target_dir: &Path) -> Result<(), String> {
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).ok();
    }
    fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;

    let file_name = archive.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let is_xz = file_name.ends_with(".xz");
    let is_gz = file_name.ends_with(".gz") || file_name.ends_with(".tgz");

    if !is_xz && !is_gz {
        return Err(format!("Unsupported archive extension: {file_name}"));
    }

    let file = fs::File::open(archive)
        .map_err(|e| format!("Failed to open archive file {}: {e}", archive.display()))?;

    if is_xz {
        let xz_decoder = lzma_rust2::XzReader::new(file, true);
        let mut tar_archive = tar::Archive::new(xz_decoder);
        unpack_and_verify(&mut tar_archive, target_dir)?;
    } else {
        let gz_decoder = flate2::read::GzDecoder::new(file);
        let mut tar_archive = tar::Archive::new(gz_decoder);
        unpack_and_verify(&mut tar_archive, target_dir)?;
    }

    // If there is a single nested folder inside target_dir, move its children out
    let entries = fs::read_dir(target_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    if entries.len() == 1 && entries[0].path().is_dir() {
        let nested_dir = entries[0].path();
        let nested_entries = fs::read_dir(&nested_dir).map_err(|e| e.to_string())?;
        for entry in nested_entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            fs::rename(entry.path(), target_dir.join(name)).map_err(|e| e.to_string())?;
        }
        fs::remove_dir(&nested_dir).ok();
    }
    Ok(())
}

fn unpack_and_verify<R: std::io::Read>(
    archive: &mut tar::Archive<R>,
    target_dir: &Path,
) -> Result<(), String> {
    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read archive entries: {e}"))?;

    for entry_res in entries {
        let mut entry = entry_res.map_err(|e| format!("Failed to read archive entry: {e}"))?;
        let entry_path = entry
            .path()
            .map_err(|e| format!("Failed to get entry path: {e}"))?
            .to_path_buf();

        if !is_path_safe(&entry_path) {
            return Err(format!(
                "Path traversal detected in archive entry: {}",
                entry_path.display()
            ));
        }

        entry
            .unpack_in(target_dir)
            .map_err(|e| format!("Failed to unpack entry {}: {e}", entry_path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tar_gz_and_flatten() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("archive-test-{}", nanos));
        fs::create_dir_all(&temp_dir).unwrap();

        let archive_path = temp_dir.join("test.tar.gz");
        let file = fs::File::create(&archive_path).unwrap();
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(enc);

        let mut header = tar::Header::new_gnu();
        header.set_path("nested_dir/").unwrap();
        header.set_entry_type(tar::EntryType::Directory);
        header.set_size(0);
        header.set_cksum();
        builder.append(&header, &[][..]).unwrap();

        let mut header2 = tar::Header::new_gnu();
        header2.set_path("nested_dir/file.txt").unwrap();
        let content = b"hello archive";
        header2.set_size(content.len() as u64);
        header2.set_entry_type(tar::EntryType::Regular);
        header2.set_cksum();
        builder.append(&header2, &content[..]).unwrap();

        builder.into_inner().unwrap().finish().unwrap();

        let target_dir = temp_dir.join("target");
        extract_archive(&archive_path, &target_dir).unwrap();

        let extracted_file = target_dir.join("file.txt");
        assert!(extracted_file.is_file());
        let data = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(data, "hello archive");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_extract_tar_xz_and_flatten() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("archive-test-xz-{}", nanos));
        fs::create_dir_all(&temp_dir).unwrap();

        let archive_path = temp_dir.join("test.tar.xz");
        let file = fs::File::create(&archive_path).unwrap();

        // Let's use XzWriter to compress the tarball
        let enc = lzma_rust2::XzWriter::new(file, lzma_rust2::XzOptions::default()).unwrap();
        let mut builder = tar::Builder::new(enc);

        let mut header = tar::Header::new_gnu();
        header.set_path("nested_dir/").unwrap();
        header.set_entry_type(tar::EntryType::Directory);
        header.set_size(0);
        header.set_cksum();
        builder.append(&header, &[][..]).unwrap();

        let mut header2 = tar::Header::new_gnu();
        header2.set_path("nested_dir/file.txt").unwrap();
        let content = b"hello xz archive";
        header2.set_size(content.len() as u64);
        header2.set_entry_type(tar::EntryType::Regular);
        header2.set_cksum();
        builder.append(&header2, &content[..]).unwrap();

        builder.into_inner().unwrap().finish().unwrap();

        let target_dir = temp_dir.join("target");
        extract_archive(&archive_path, &target_dir).unwrap();

        let extracted_file = target_dir.join("file.txt");
        assert!(extracted_file.is_file());
        let data = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(data, "hello xz archive");

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_extract_path_traversal() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("archive-test-{}", nanos));
        fs::create_dir_all(&temp_dir).unwrap();

        let archive_path = temp_dir.join("evil.tar.gz");
        let file = fs::File::create(&archive_path).unwrap();
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(enc);

        let mut header = tar::Header::new_gnu();
        // Manually write the raw path to bypass set_path validation
        let bytes = header.as_mut_bytes();
        let path_str = "../evil.txt";
        bytes[..path_str.len()].copy_from_slice(path_str.as_bytes());
        let content = b"evil";
        header.set_size(content.len() as u64);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        builder.append(&header, &content[..]).unwrap();

        builder.into_inner().unwrap().finish().unwrap();

        let target_dir = temp_dir.join("target");
        let result = extract_archive(&archive_path, &target_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Path traversal detected"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_extract_unsupported_format() {
        let archive_path = Path::new("unsupported.zip");
        let target_dir = Path::new("target");
        let result = extract_archive(archive_path, target_dir);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Unsupported archive extension")
        );
    }
}
