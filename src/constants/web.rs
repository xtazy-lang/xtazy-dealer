macro_rules! dealer_url {
    ($path:expr) => {
        concat!("https://dealer.xtazy.dev", $path)
    };
}

pub(crate) const BASE_URL: &str = dealer_url!("");
pub(crate) const VERSION_TXT_URL: &str = dealer_url!("/version.txt");
pub(crate) const TARGETS_TSV_URL: &str = dealer_url!("/targets.tsv");
pub(crate) const TARGETS_TSV_XSIGFILE_URL: &str = dealer_url!("/targets.tsv.xsigfile");
pub(crate) const XTAZY_LATEST_URL: &str = dealer_url!("/xtazy");

pub(crate) fn dealer_archive_url(version: &str, suffix: &str) -> String {
    format!(
        "{}/dealer/v{}/dealer-{}-{}.tar.gz",
        BASE_URL, version, version, suffix
    )
}

pub(crate) fn xtazy_parts_url(version: &str) -> String {
    format!("{}/xtazy/v{}/xtazy.parts", BASE_URL, version)
}

pub(crate) fn xtazy_parts_xsigfile_url(version: &str) -> String {
    format!("{}/xtazy/v{}/xtazy.parts.xsigfile", BASE_URL, version)
}

pub(crate) fn piko_targets_tsv_url(version: &str) -> String {
    format!("{}/piko/v{}/targets.tsv", BASE_URL, version)
}

pub(crate) fn piko_targets_tsv_xsigfile_url(version: &str) -> String {
    format!("{}/piko/v{}/targets.tsv.xsigfile", BASE_URL, version)
}

pub(crate) fn piko_archive_url(version: &str, suffix: &str) -> String {
    format!(
        "{}/piko/v{}/piko-{}-{}.tar.gz",
        BASE_URL, version, version, suffix
    )
}

pub(crate) fn rusttime_archive_url(version: &str) -> String {
    format!(
        "{}/rusttime/v{}/rusttime-{}.tar.gz",
        BASE_URL, version, version
    )
}

pub(crate) fn std_archive_url(version: &str) -> String {
    format!("{}/std/v{}/std-{}.tar.gz", BASE_URL, version, version)
}

pub(crate) fn package_latest_url(name: &str) -> String {
    format!("{}/package/{}/latest", BASE_URL, name)
}

pub(crate) fn package_versions_url(name: &str) -> String {
    format!("{}/package/{}", BASE_URL, name)
}

pub(crate) fn package_version_url(name: &str, version: &str) -> String {
    format!("{}/package/{}/{}", BASE_URL, name, version)
}

pub(crate) fn rust_manifest_url(version: &str) -> String {
    format!(
        "https://static.rust-lang.org/dist/channel-rust-{}.toml",
        version
    )
}
