use crate::state::{DealerState, parse_xtazy_parts, rust_backend_id_for_version};
use std::fs;

pub(crate) fn has_complete_toolchain(state: &DealerState, version: &str) -> bool {
    let tdir = state.toolchain_dir(version);
    let parts_file = tdir.join(crate::constants::files::XTAZY_PARTS);
    if !parts_file.is_file() {
        return false;
    }

    let content = match fs::read_to_string(&parts_file) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let parsed = match parse_xtazy_parts(&content) {
        Ok(p) => p,
        Err(_) => return false,
    };

    let piko_exe = state
        .dealer_home
        .join(crate::constants::dirs::PIKO_COMPONENT_DIR)
        .join(&parsed.piko_version)
        .join(format!(
            "{}{}",
            crate::constants::executables::EXE_PIKO,
            std::env::consts::EXE_SUFFIX
        ));
    let rusttime_dir = state
        .dealer_home
        .join(crate::constants::dirs::RUSTTIME_DIR)
        .join(&parsed.rusttime_version);
    let std_dir = state
        .dealer_home
        .join(crate::constants::dirs::STD_DIR)
        .join(&parsed.std_version);

    let backend_id = rust_backend_id_for_version(&parsed.rust_version);
    let rust_dir = state.rust_backend_dir(&backend_id);
    let cargo_exe = rust_dir.join(crate::constants::dirs::BIN_DIR).join(format!(
        "{}{}",
        crate::constants::executables::EXE_CARGO,
        std::env::consts::EXE_SUFFIX
    ));
    let rustc_exe = rust_dir.join(crate::constants::dirs::BIN_DIR).join(format!(
        "{}{}",
        crate::constants::executables::EXE_RUSTC,
        std::env::consts::EXE_SUFFIX
    ));

    piko_exe.is_file()
        && rusttime_dir.is_dir()
        && std_dir.is_dir()
        && cargo_exe.is_file()
        && rustc_exe.is_file()
        && rust_dir.join("lib").join("rustlib").is_dir()
}
