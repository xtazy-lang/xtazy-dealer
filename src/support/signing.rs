use std::fs;
use std::path::Path;

pub(crate) fn verify_xsig_signature(
    master_public_bytes: &[u8],
    file_to_verify: &Path,
    signature_file: &Path,
) -> Result<(), String> {
    let master_public_record =
        std::str::from_utf8(master_public_bytes).map_err(|e| e.to_string())?;
    let file_bytes = fs::read(file_to_verify).map_err(|e| e.to_string())?;
    let xsigfile_record = fs::read_to_string(signature_file).map_err(|e| e.to_string())?;

    match xsig::verify_file_with_master_public(master_public_record, &file_bytes, &xsigfile_record)
    {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("{err:?}")),
    }
}
