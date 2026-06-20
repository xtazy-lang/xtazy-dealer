use crate::compiler::CompilerBackend;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct ProjectRoot {
    pub(crate) root_dir: PathBuf,
    pub(crate) root_file: PathBuf,
    pub(crate) package_name: String,
}

pub(crate) fn validate_project_root(path: &Path) -> Result<ProjectRoot, String> {
    let root_dir = fs::canonicalize(path).map_err(|error| {
        format!(
            "failed to read project folder '{}': {error}",
            path.display()
        )
    })?;
    if !root_dir.is_dir() {
        return Err(format!(
            "expected a project folder, found '{}'",
            root_dir.display()
        ));
    }

    let app = root_dir.join("app.x");
    let package = root_dir.join("package.x");
    match (app.is_file(), package.is_file()) {
        (true, false) => Ok(project_root(root_dir, app)),
        (false, true) => Ok(project_root(root_dir, package)),
        (false, false) => Err(format!(
            "'{}' is not a valid Xtazy project: expected app.x or package.x",
            root_dir.display()
        )),
        (true, true) => Err(format!(
            "'{}' is ambiguous: app.x and package.x cannot exist together",
            root_dir.display()
        )),
    }
}

fn project_root(root_dir: PathBuf, root_file: PathBuf) -> ProjectRoot {
    let package_name = root_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("xtazy_project")
        .to_string();

    ProjectRoot {
        root_dir,
        root_file,
        package_name,
    }
}

pub(crate) fn resolve_dependencies(
    project: &ProjectRoot,
    compiler: &dyn CompilerBackend,
) -> Result<HashMap<String, PathBuf>, String> {
    let mut resolved = HashMap::new();
    let mut path_stack = Vec::new();

    let root_meta = compiler
        .metadata(&project.root_file)
        .map_err(|e| format!("Failed to read project root metadata: {e}"))?;

    resolve_recursive(
        &root_meta.dependencies,
        &project.root_dir,
        compiler,
        &mut resolved,
        &mut path_stack,
    )?;

    Ok(resolved)
}

fn resolve_recursive(
    dependencies: &[crate::compiler::MetadataDependency],
    base_dir: &Path,
    compiler: &dyn CompilerBackend,
    resolved: &mut HashMap<String, PathBuf>,
    path_stack: &mut Vec<String>,
) -> Result<(), String> {
    for dep in dependencies {
        let dep_path = match dep.source_type.as_str() {
            "local_path" => {
                let p = Path::new(&dep.arg1);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    base_dir.join(p)
                }
            }
            "registry_exact" => {
                let workspace_root = crate::workspace_root();
                let index_pkg = workspace_root.join("package").join(&dep.name);
                if index_pkg.is_dir() {
                    index_pkg
                } else {
                    crate::state::DealerState::from_process_env(&workspace_root)
                        .cache_dir()
                        .join("package")
                        .join(&dep.name)
                }
            }
            _ => {
                return Err(format!(
                    "Dependency source type '{}' with values '{}'{} is not supported in resolver MVP yet",
                    dep.source_type,
                    dep.arg1,
                    dep.arg2
                        .as_deref()
                        .map(|value| format!(", '{}'", value))
                        .unwrap_or_default()
                ));
            }
        };

        let dep_path = fs::canonicalize(&dep_path).map_err(|e| {
            format!(
                "Failed to locate dependency '{}' at '{}': {e}",
                dep.name,
                dep_path.display()
            )
        })?;

        let package_x = dep_path.join("package.x");
        if !package_x.is_file() {
            return Err(format!(
                "Dependency '{}' at '{}' is invalid: missing package.x",
                dep.name,
                dep_path.display()
            ));
        }

        let dep_meta = compiler
            .metadata(&package_x)
            .map_err(|e| format!("Failed to read metadata for dependency '{}': {e}", dep.name))?;

        if dep_meta.project_type != "package" {
            return Err(format!(
                "Dependency '{}' resolved to '{}', which is an app, not a package",
                dep.name,
                dep_path.display()
            ));
        }

        if dep_meta.name != dep.name {
            return Err(format!(
                "Package name mismatch: declared dependency is '{}', but resolved package claims name '{}'",
                dep.name, dep_meta.name
            ));
        }

        if path_stack.contains(&dep.name) {
            return Err(format!("Circular dependency detected: {:?}", path_stack));
        }

        if let Some(existing_path) = resolved.get(&dep.name) {
            if existing_path != &dep_path {
                return Err(format!(
                    "Conflict detected: package '{}' is resolved to two different paths: '{}' and '{}'",
                    dep.name,
                    existing_path.display(),
                    dep_path.display()
                ));
            }
            continue;
        }

        resolved.insert(dep.name.clone(), dep_path.clone());
        path_stack.push(dep.name.clone());

        resolve_recursive(
            &dep_meta.dependencies,
            &dep_path,
            compiler,
            resolved,
            path_stack,
        )?;

        path_stack.pop();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempProject;

    #[test]
    fn validate_project_accepts_app_root() {
        let temp = TempProject::new("app-root");
        fs::write(temp.path().join("app.x"), "app Main\n").expect("app root should be written");

        let project = validate_project_root(temp.path()).expect("app.x root should be valid");
        let expected_root = fs::canonicalize(temp.path()).expect("temp root should canonicalize");
        let expected_name = temp.path().file_name().unwrap().to_string_lossy();

        assert_eq!(project.root_file, expected_root.join("app.x"));
        assert_eq!(project.package_name, expected_name);
    }

    #[test]
    fn validate_project_accepts_package_root() {
        let temp = TempProject::new("package-root");
        fs::write(temp.path().join("package.x"), "entity Thing\n")
            .expect("package root should be written");

        let project = validate_project_root(temp.path()).expect("package.x root should be valid");
        let expected_root = fs::canonicalize(temp.path()).expect("temp root should canonicalize");

        assert_eq!(project.root_file, expected_root.join("package.x"));
    }

    #[test]
    fn validate_project_rejects_missing_root_file() {
        let temp = TempProject::new("missing-root");

        let error = validate_project_root(temp.path()).expect_err("missing root should fail");

        assert!(error.contains("expected app.x or package.x"));
    }

    #[test]
    fn validate_project_rejects_ambiguous_root_files() {
        let temp = TempProject::new("ambiguous-root");
        fs::write(temp.path().join("app.x"), "app Main\n").expect("app root should be written");
        fs::write(temp.path().join("package.x"), "entity Thing\n")
            .expect("package root should be written");

        let error = validate_project_root(temp.path()).expect_err("ambiguous root should fail");

        assert!(error.contains("app.x and package.x cannot exist together"));
    }
}
