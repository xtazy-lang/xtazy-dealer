pub(crate) mod components;
pub(crate) mod dirs;
pub(crate) mod executables;
pub(crate) mod files;
pub(crate) mod metadata;
pub(crate) mod protocol;
pub(crate) mod web;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_routes_builders() {
        assert_eq!(web::BASE_URL, "https://dealer.xtazy.dev");
        assert_eq!(web::VERSION_TXT_URL, "https://dealer.xtazy.dev/version.txt");
        assert_eq!(web::TARGETS_TSV_URL, "https://dealer.xtazy.dev/targets.tsv");
        assert_eq!(
            web::TARGETS_TSV_XSIGFILE_URL,
            "https://dealer.xtazy.dev/targets.tsv.xsigfile"
        );
        assert_eq!(web::XTAZY_LATEST_URL, "https://dealer.xtazy.dev/xtazy");

        assert_eq!(
            web::dealer_archive_url("1.0.0", "x86_64"),
            "https://dealer.xtazy.dev/dealer/v1.0.0/dealer-1.0.0-x86_64.tar.gz"
        );
        assert_eq!(
            web::xtazy_parts_url("0.1.0"),
            "https://dealer.xtazy.dev/xtazy/v0.1.0/xtazy.parts"
        );
        assert_eq!(
            web::xtazy_parts_xsigfile_url("0.1.0"),
            "https://dealer.xtazy.dev/xtazy/v0.1.0/xtazy.parts.xsigfile"
        );
        assert_eq!(
            web::piko_targets_tsv_url("0.2.0"),
            "https://dealer.xtazy.dev/piko/v0.2.0/targets.tsv"
        );
        assert_eq!(
            web::piko_targets_tsv_xsigfile_url("0.2.0"),
            "https://dealer.xtazy.dev/piko/v0.2.0/targets.tsv.xsigfile"
        );
        assert_eq!(
            web::piko_archive_url("0.2.0", "arm64"),
            "https://dealer.xtazy.dev/piko/v0.2.0/piko-0.2.0-arm64.tar.gz"
        );
        assert_eq!(
            web::rusttime_archive_url("1.80"),
            "https://dealer.xtazy.dev/rusttime/v1.80/rusttime-1.80.tar.gz"
        );
        assert_eq!(
            web::std_archive_url("2.0.0"),
            "https://dealer.xtazy.dev/std/v2.0.0/std-2.0.0.tar.gz"
        );
        assert_eq!(
            web::package_latest_url("core"),
            "https://dealer.xtazy.dev/package/core/latest"
        );
        assert_eq!(
            web::package_version_url("core", "1.2.0"),
            "https://dealer.xtazy.dev/package/core/1.2.0"
        );
        assert_eq!(
            web::rust_manifest_url("1.80.0"),
            "https://static.rust-lang.org/dist/channel-rust-1.80.0.toml"
        );
    }

    #[test]
    fn test_protocol_constants() {
        assert_eq!(protocol::PROTOCOL_COMPILER_DIR, "piko");
        assert_eq!(protocol::PROTOCOL_REQUEST_DIR, "request");
        assert_eq!(protocol::PROTOCOL_RESULT_DIR, "result");
        assert_eq!(protocol::PROTOCOL_MODE, "mode");
        assert_eq!(protocol::PROTOCOL_ENTRY_FILE, "entry_file");
        assert_eq!(protocol::PROTOCOL_PROJECT_ROOT, "project_root");
        assert_eq!(protocol::PROTOCOL_PROJECT_NAME, "project_name");
        assert_eq!(protocol::PROTOCOL_COLOR, "color");
        assert_eq!(
            protocol::PROTOCOL_RESOLVED_PACKAGES,
            "resolved_packages.tsv"
        );
        assert_eq!(protocol::PROTOCOL_RUST_OUTPUT_DIR, "rust_output_dir");
        assert_eq!(
            protocol::PROTOCOL_GENERATED_PACKAGE_NAME,
            "generated_package_name"
        );
        assert_eq!(protocol::PROTOCOL_RUSTTIME_PATH, "rusttime_path");
        assert_eq!(protocol::PROTOCOL_STATUS, "status");
        assert_eq!(protocol::PROTOCOL_DIAGNOSTICS, "diagnostics");

        assert_eq!(protocol::STATUS_OK, "ok");
        assert_eq!(protocol::STATUS_WARNING, "warning");
        assert_eq!(protocol::STATUS_ERROR, "error");
    }
}
