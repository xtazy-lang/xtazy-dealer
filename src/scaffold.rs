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
        InitKind::App => (app, format!("app {name} 1.0.0\n")),
        InitKind::Package => (package, format!("package {name} 1.0.0\n")),
    };

    fs::write(&root_file, &content).map_err(|error| DealerError::io(&root_file, error))?;
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
        format!("xtazy{ident}")
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

        // Check exact content
        let content = fs::read_to_string(&root).unwrap();
        let expected_name = project_name(temp.path());
        assert_eq!(content, format!("app {expected_name} 1.0.0\n"));

        // Verify parseability
        let decl = crate::project::dealer_block::parse_project_file(&content).unwrap();
        assert!(decl.is_app);
        assert_eq!(decl.name, expected_name);
        assert_eq!(decl.version, "1.0.0");
    }

    #[test]
    fn init_package_creates_package_root_only() {
        let temp = TempProject::new("init-pkg");

        let root = init_project(InitKind::Package, temp.path()).expect("package init should pass");

        assert_eq!(root, temp.path().join("package.x"));
        assert!(temp.path().join("package.x").is_file());
        assert!(!temp.path().join("app.x").exists());

        // Check exact content
        let content = fs::read_to_string(&root).unwrap();
        let expected_name = project_name(temp.path());
        assert_eq!(content, format!("package {expected_name} 1.0.0\n"));

        // Verify parseability
        let decl = crate::project::dealer_block::parse_project_file(&content).unwrap();
        assert!(!decl.is_app);
        assert_eq!(decl.name, expected_name);
        assert_eq!(decl.version, "1.0.0");
    }

    #[test]
    fn init_refuses_existing_root() {
        let temp = TempProject::new("init-existing");
        fs::write(temp.path().join("app.x"), "app Existing 1.0.0\n")
            .expect("app.x should be written");

        let error =
            init_project(InitKind::Package, temp.path()).expect_err("existing root should fail");

        assert!(
            error
                .to_string()
                .contains("already contains app.x or package.x")
        );
    }
}
