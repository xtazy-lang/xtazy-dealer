use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{DealerError, DealerResult};
use crate::project::dealer_block::parse_project_file;

#[derive(Debug, Clone)]
pub(crate) struct ProjectRoot {
    pub(crate) root_dir: PathBuf,
    pub(crate) root_file: PathBuf,
    pub(crate) project_name: String,
    pub(crate) _project_version: String,
}

pub(crate) fn validate_project_root(dir: &Path) -> DealerResult<ProjectRoot> {
    let root_dir = fs::canonicalize(dir).map_err(|e| DealerError::io(dir, e))?;

    let app = root_dir.join("app.x");
    let package = root_dir.join("package.x");
    let root_file = match (app.is_file(), package.is_file()) {
        (true, false) => app,
        (false, true) => package,
        (false, false) => {
            return Err(DealerError::Project(format!(
                "'{}' is not a valid xtazy project: expected app.x or package.x",
                root_dir.display()
            )));
        }
        (true, true) => {
            return Err(DealerError::Project(format!(
                "'{}' is ambiguous: app.x and package.x cannot exist together",
                root_dir.display()
            )));
        }
    };

    let content = fs::read_to_string(&root_file).map_err(|e| DealerError::io(&root_file, e))?;
    let decl = parse_project_file(&content)?;

    Ok(ProjectRoot {
        root_dir,
        root_file,
        project_name: decl.name,
        _project_version: decl.version,
    })
}
