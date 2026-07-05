use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::compiler_contract::contract::{BuildRequestArgs, CompilerBackend};
use crate::error::{DealerError, DealerResult};
use crate::project::ProjectRoot;

#[derive(Debug, Clone)]
pub(crate) struct ExecutableCompilerBackend {
    pub(crate) compiler_path: PathBuf,
}

impl ExecutableCompilerBackend {
    fn run_compiler_protocol(
        &self,
        project_root: &Path,
        mode: &str,
        entry_file: &Path,
        project_name: &str,
        deps: Option<&HashMap<String, PathBuf>>,
        build_args: Option<BuildRequestArgs>,
    ) -> DealerResult<()> {
        let protocol_dir = project_root
            .join(crate::constants::dirs::PROJECT_DEALER_DIR)
            .join(crate::constants::dirs::PROJECT_XTAZY_DIR)
            .join(crate::constants::protocol::PROTOCOL_COMPILER_DIR);
        let request_dir = protocol_dir.join(crate::constants::protocol::PROTOCOL_REQUEST_DIR);
        let result_dir = protocol_dir.join(crate::constants::protocol::PROTOCOL_RESULT_DIR);

        if request_dir.exists() {
            fs::remove_dir_all(&request_dir).ok();
        }
        if result_dir.exists() {
            fs::remove_dir_all(&result_dir).ok();
        }
        fs::create_dir_all(&request_dir).map_err(|e| DealerError::io(&request_dir, e))?;

        fs::write(
            request_dir.join(crate::constants::protocol::PROTOCOL_MODE),
            format!("{}\n", mode),
        )
        .map_err(|e| {
            DealerError::io(
                request_dir.join(crate::constants::protocol::PROTOCOL_MODE),
                e,
            )
        })?;
        fs::write(
            request_dir.join(crate::constants::protocol::PROTOCOL_ENTRY_FILE),
            format!("{}\n", entry_file.display()),
        )
        .map_err(|e| {
            DealerError::io(
                request_dir.join(crate::constants::protocol::PROTOCOL_ENTRY_FILE),
                e,
            )
        })?;
        fs::write(
            request_dir.join(crate::constants::protocol::PROTOCOL_PROJECT_ROOT),
            format!("{}\n", project_root.display()),
        )
        .map_err(|e| {
            DealerError::io(
                request_dir.join(crate::constants::protocol::PROTOCOL_PROJECT_ROOT),
                e,
            )
        })?;
        fs::write(
            request_dir.join(crate::constants::protocol::PROTOCOL_PROJECT_NAME),
            format!("{}\n", project_name),
        )
        .map_err(|e| {
            DealerError::io(
                request_dir.join(crate::constants::protocol::PROTOCOL_PROJECT_NAME),
                e,
            )
        })?;
        fs::write(
            request_dir.join(crate::constants::protocol::PROTOCOL_COLOR),
            "auto\n",
        )
        .map_err(|e| {
            DealerError::io(
                request_dir.join(crate::constants::protocol::PROTOCOL_COLOR),
                e,
            )
        })?;

        if let Some(deps_map) = deps {
            let mut tsv_content = String::new();
            // Sort keys to ensure deterministic output for testing/debugging
            let mut keys: Vec<&String> = deps_map.keys().collect();
            keys.sort();
            for name in keys {
                let path = &deps_map[name];
                tsv_content.push_str(&format!("{}\t{}\n", name, path.display()));
            }
            fs::write(
                request_dir.join(crate::constants::protocol::PROTOCOL_RESOLVED_PACKAGES),
                tsv_content,
            )
            .map_err(|e| {
                DealerError::io(
                    request_dir.join(crate::constants::protocol::PROTOCOL_RESOLVED_PACKAGES),
                    e,
                )
            })?;
        }

        if let Some(bargs) = build_args {
            fs::write(
                request_dir.join(crate::constants::protocol::PROTOCOL_RUST_OUTPUT_DIR),
                format!("{}\n", bargs.rust_output_dir.display()),
            )
            .map_err(|e| {
                DealerError::io(
                    request_dir.join(crate::constants::protocol::PROTOCOL_RUST_OUTPUT_DIR),
                    e,
                )
            })?;
            fs::write(
                request_dir.join(crate::constants::protocol::PROTOCOL_GENERATED_PACKAGE_NAME),
                format!("{}\n", bargs.generated_package_name),
            )
            .map_err(|e| {
                DealerError::io(
                    request_dir.join(crate::constants::protocol::PROTOCOL_GENERATED_PACKAGE_NAME),
                    e,
                )
            })?;
            fs::write(
                request_dir.join(crate::constants::protocol::PROTOCOL_RUSTTIME_PATH),
                format!("{}\n", bargs.rusttime_path.display()),
            )
            .map_err(|e| {
                DealerError::io(
                    request_dir.join(crate::constants::protocol::PROTOCOL_RUSTTIME_PATH),
                    e,
                )
            })?;
        }

        let output = Command::new(&self.compiler_path)
            .arg("--request")
            .arg(&request_dir)
            .arg("--result")
            .arg(&result_dir)
            .output()
            .map_err(|e| DealerError::Compiler(format!("failed to start πko process: {e}")))?;

        let status_file = result_dir.join(crate::constants::protocol::PROTOCOL_STATUS);
        let diagnostics_file = result_dir.join(crate::constants::protocol::PROTOCOL_DIAGNOSTICS);

        if !status_file.is_file() {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(DealerError::Compiler(format!(
                    "πko exited with status {}\nstdout:\n{}\nstderr:\n{}",
                    output.status, stdout, stderr
                )));
            }
            return Err(DealerError::Compiler(
                "πko exited without writing result/status file".to_string(),
            ));
        }

        let status = fs::read_to_string(&status_file)
            .map_err(|e| DealerError::io(&status_file, e))?
            .trim()
            .to_string();
        let diagnostics = if diagnostics_file.is_file() {
            fs::read_to_string(&diagnostics_file).unwrap_or_default()
        } else {
            String::new()
        };

        if !diagnostics.is_empty() {
            print!("{}", diagnostics);
        }

        if status == crate::constants::protocol::STATUS_OK
            || status == crate::constants::protocol::STATUS_WARNING
        {
            Ok(())
        } else if status == crate::constants::protocol::STATUS_ERROR {
            Err(DealerError::Compiler("πko reported errors".to_string()))
        } else {
            Err(DealerError::Compiler(format!(
                "πko returned unknown status: '{status}'"
            )))
        }
    }
}

impl CompilerBackend for ExecutableCompilerBackend {
    fn check(&self, project: &ProjectRoot, deps: &HashMap<String, PathBuf>) -> DealerResult<()> {
        self.run_compiler_protocol(
            &project.root_dir,
            "check",
            &project.root_file,
            &project.project_name,
            Some(deps),
            None,
        )
    }

    fn build(
        &self,
        project: &ProjectRoot,
        deps: &HashMap<String, PathBuf>,
        output_dir: &Path,
        rusttime_path: &Path,
    ) -> DealerResult<()> {
        let build_args = BuildRequestArgs {
            rust_output_dir: output_dir.to_path_buf(),
            generated_package_name: crate::names::sanitize_package_name(&project.project_name),
            rusttime_path: rusttime_path.to_path_buf(),
        };
        self.run_compiler_protocol(
            &project.root_dir,
            "build",
            &project.root_file,
            &project.project_name,
            Some(deps),
            Some(build_args),
        )
    }

    fn test(&self, project: &ProjectRoot, deps: &HashMap<String, PathBuf>) -> DealerResult<()> {
        self.run_compiler_protocol(
            &project.root_dir,
            "test",
            &project.root_file,
            &project.project_name,
            Some(deps),
            None,
        )
    }

    fn fmt(
        &self,
        project_root: &Path,
        entry_file: &Path,
        project_name: &str,
        check: bool,
    ) -> DealerResult<()> {
        let mode = if check { "fmt-check" } else { "fmt" };
        self.run_compiler_protocol(project_root, mode, entry_file, project_name, None, None)
    }
}
