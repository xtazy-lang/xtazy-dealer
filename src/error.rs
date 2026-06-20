use std::io;
use std::path::PathBuf;
use std::process::Output;

pub(crate) type DealerResult<T> = Result<T, DealerError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum DealerError {
    #[error("{0}")]
    PackageResolution(String),
    #[error("{0}")]
    Compiler(String),
    #[error("{0}")]
    Backend(String),
    #[error("{feature} is recognized but not implemented yet")]
    NotImplemented { feature: String },
    #[error("I/O error at '{}': {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

impl DealerError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

pub(crate) fn process_output_message(output: Output, label: &str) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut message = String::new();
    if !stdout.trim().is_empty() {
        message.push_str(&stdout);
    }
    if !stderr.trim().is_empty() {
        message.push_str(&stderr);
    }
    if message.is_empty() {
        message.push_str(&format!("{label} exited with status {}", output.status));
    }
    message
}
