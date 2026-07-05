use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(crate) fn compute_file_sha256(path: &Path) -> Result<String, String> {
    if !path.is_file() {
        return Err(format!("File not found: {}", path.display()));
    }

    let mut file =
        File::open(path).map_err(|e| format!("Failed to open file {}: {e}", path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file {}: {e}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compute_file_sha256_known_vector() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("checksum-test-{}", nanos));
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let hash = compute_file_sha256(&file_path).unwrap();
        // SHA-256 of "hello world" is:
        // b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_compute_file_sha256_missing_file() {
        let path = Path::new("non_existent_file.xyz");
        let result = compute_file_sha256(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }
}
