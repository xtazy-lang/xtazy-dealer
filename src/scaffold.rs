use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::InitKind;
use crate::error::{DealerError, DealerResult};

pub(crate) fn init_project(kind: InitKind, path: &Path) -> DealerResult<PathBuf> {
    fs::create_dir_all(path).map_err(|error| DealerError::io(path, error))?;

    let app = path.join("app.x");
    let package = path.join("package.x");
    if app.exists() || package.exists() {
        return Err(DealerError::Backend(format!(
            "project root '{}' already contains app.x or package.x",
            path.display()
        )));
    }

    let name = project_name(path);
    let (root_file, content) = match kind {
        InitKind::App => (
            app,
            format!("app {name}\n\tterminal.log\n\t\tmessage: \"Hello from Xtazy\"\n"),
        ),
        InitKind::Package => (package, format!("package {name}\n\tdeliver\n")),
    };

    fs::write(&root_file, content).map_err(|error| DealerError::io(&root_file, error))?;
    Ok(root_file)
}

fn project_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && *name != ".")
        .map(sanitize_xtazy_ident)
        .unwrap_or_else(|| "NewProject".to_string())
}

fn sanitize_xtazy_ident(value: &str) -> String {
    let mut ident = String::new();
    let mut upper_next = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if upper_next {
                ident.push(ch.to_ascii_uppercase());
                upper_next = false;
            } else {
                ident.push(ch);
            }
        } else {
            upper_next = true;
        }
    }
    if ident.is_empty() || ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        format!("Xtazy{ident}")
    } else {
        ident
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempProject;

    #[test]
    fn init_app_creates_app_root_only() {
        let temp = TempProject::new("init-app");

        let root = init_project(InitKind::App, temp.path()).expect("app init should pass");

        assert_eq!(root, temp.path().join("app.x"));
        assert!(temp.path().join("app.x").is_file());
        assert!(!temp.path().join("package.x").exists());
    }

    #[test]
    fn init_refuses_existing_root() {
        let temp = TempProject::new("init-existing");
        fs::write(temp.path().join("app.x"), "app Existing\n").expect("app.x should be written");

        let error =
            init_project(InitKind::Package, temp.path()).expect_err("existing root should fail");

        assert!(
            error
                .to_string()
                .contains("already contains app.x or package.x")
        );
    }
}
