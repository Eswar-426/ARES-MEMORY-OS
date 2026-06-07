use ares_core::AresError;
use std::path::Path;

/// Compute the Blake3 hash of a file's contents.
/// Returns the hash as a lowercase hex string.
pub fn hash_file(path: &Path) -> Result<String, AresError> {
    let contents = std::fs::read(path).map_err(AresError::Io)?;
    let hash = blake3::hash(&contents);
    Ok(hash.to_hex().to_string())
}

/// Compute the Blake3 hash of a byte slice.
pub fn hash_bytes(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn hash_file_returns_hex_string() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello world").unwrap();
        let hash = hash_file(f.path()).unwrap();
        assert_eq!(hash.len(), 64); // Blake3 = 256 bits = 64 hex chars
    }

    #[test]
    fn same_content_same_hash() {
        assert_eq!(hash_bytes(b"test"), hash_bytes(b"test"));
    }

    #[test]
    fn different_content_different_hash() {
        assert_ne!(hash_bytes(b"a"), hash_bytes(b"b"));
    }
}
