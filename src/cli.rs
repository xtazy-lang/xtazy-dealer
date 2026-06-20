use clap::{Args, CommandFactory, Parser, Subcommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommandKind {
    Version,
    Check {
        project: String,
    },
    Fmt {
        project: String,
        check: bool,
    },
    Install {
        package: Option<String>,
        project: String,
    },
    Build {
        project: String,
        mode: BuildMode,
    },
    Run {
        project: String,
        mode: BuildMode,
    },
    Clean {
        project: String,
    },
    Init {
        kind: InitKind,
        path: Option<String>,
    },
    Xtazy {
        subcommand: XtazySubcommand,
    },
    SelfUpdate,
    Doctor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildMode {
    Dev,
    Prod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InitKind {
    App,
    Package,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum XtazySubcommand {
    Install { version: Option<String> },
    Update,
    AutoUpdate { action: Option<AutoUpdateAction> },
    UseVersion { version: String },
    Active,
    List,
    Remove { version: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AutoUpdateAction {
    Off,
    Status,
}

#[derive(Parser, Debug)]
#[command(
    name = "dealer",
    about = "Xtazy project orchestrator",
    disable_version_flag = true
)]
struct DealerCli {
    #[command(subcommand)]
    command: DealerCommand,
}

#[derive(Subcommand, Debug)]
enum DealerCommand {
    Check(ProjectArg),
    Fmt(FmtArgs),
    Install(InstallArgs),
    Build(BuildArgs),
    Run(BuildArgs),
    Clean(ProjectArg),
    Init(InitArgs),
    Xtazy(XtazyArgs),
    #[command(name = "self")]
    SelfCommand(SelfArgs),
    Doctor,
}

#[derive(Args, Debug)]
struct ProjectArg {
    #[arg(default_value = ".")]
    project: String,
}

#[derive(Args, Debug)]
struct FmtArgs {
    #[arg(long)]
    check: bool,
    #[arg(default_value = ".")]
    project: String,
}

#[derive(Args, Debug)]
struct InstallArgs {
    package: Option<String>,
    project: Option<String>,
}

#[derive(Args, Debug)]
struct BuildArgs {
    #[arg(long, conflicts_with = "prod")]
    dev: bool,
    #[arg(long, conflicts_with = "dev")]
    prod: bool,
    #[arg(default_value = ".")]
    project: String,
}

#[derive(Args, Debug)]
struct InitArgs {
    #[command(subcommand)]
    kind: InitCommand,
}

#[derive(Subcommand, Debug)]
enum InitCommand {
    App(InitPath),
    Package(InitPath),
}

#[derive(Args, Debug)]
struct InitPath {
    path: Option<String>,
}

#[derive(Args, Debug)]
struct XtazyArgs {
    #[command(subcommand)]
    command: XtazyCommand,
}

#[derive(Subcommand, Debug)]
enum XtazyCommand {
    Install(VersionArg),
    Update,
    AutoUpdate(AutoUpdateArgs),
    Use(RequiredVersionArg),
    Active,
    List,
    Remove(RequiredVersionArg),
}

#[derive(Args, Debug)]
struct VersionArg {
    version: Option<String>,
}

#[derive(Args, Debug)]
struct RequiredVersionArg {
    version: String,
}

#[derive(Args, Debug)]
struct AutoUpdateArgs {
    #[command(subcommand)]
    action: Option<AutoUpdateCommand>,
}

#[derive(Subcommand, Debug)]
enum AutoUpdateCommand {
    Off,
    Status,
}

#[derive(Args, Debug)]
struct SelfArgs {
    #[command(subcommand)]
    command: SelfCommand,
}

#[derive(Subcommand, Debug)]
enum SelfCommand {
    Update,
}

pub(crate) fn parse_args(args: &[String]) -> Result<CommandKind, String> {
    if args.len() == 1 {
        return Err(DealerCli::command().render_usage().to_string());
    }

    if args.get(1).map(String::as_str) == Some("--version")
        || args.get(1).map(String::as_str) == Some("-V")
    {
        return Ok(CommandKind::Version);
    }

    // Preserve the historical dev shortcut: `dealer <project>` means check that project.
    if args.len() == 2 && !args[1].starts_with('-') && !is_known_command(&args[1]) {
        return Ok(CommandKind::Check {
            project: args[1].clone(),
        });
    }

    let cli = DealerCli::try_parse_from(args).map_err(|error| error.to_string())?;
    Ok(command_from_cli(cli.command))
}

fn is_known_command(value: &str) -> bool {
    matches!(
        value,
        "check"
            | "fmt"
            | "install"
            | "build"
            | "run"
            | "clean"
            | "init"
            | "xtazy"
            | "self"
            | "doctor"
    )
}

fn command_from_cli(command: DealerCommand) -> CommandKind {
    match command {
        DealerCommand::Check(args) => CommandKind::Check {
            project: args.project,
        },
        DealerCommand::Fmt(args) => CommandKind::Fmt {
            project: args.project,
            check: args.check,
        },
        DealerCommand::Install(args) => install_command(args),
        DealerCommand::Build(args) => CommandKind::Build {
            project: args.project,
            mode: mode_from_args(args.dev, args.prod),
        },
        DealerCommand::Run(args) => CommandKind::Run {
            project: args.project,
            mode: mode_from_args(args.dev, args.prod),
        },
        DealerCommand::Clean(args) => CommandKind::Clean {
            project: args.project,
        },
        DealerCommand::Init(args) => init_command(args.kind),
        DealerCommand::Xtazy(args) => CommandKind::Xtazy {
            subcommand: xtazy_command(args.command),
        },
        DealerCommand::SelfCommand(args) => match args.command {
            SelfCommand::Update => CommandKind::SelfUpdate,
        },
        DealerCommand::Doctor => CommandKind::Doctor,
    }
}

fn install_command(args: InstallArgs) -> CommandKind {
    match (args.package, args.project) {
        (None, None) => CommandKind::Install {
            package: None,
            project: ".".to_string(),
        },
        (Some(package), None) if looks_like_project_path(&package) => CommandKind::Install {
            package: None,
            project: package,
        },
        (Some(package), None) => CommandKind::Install {
            package: Some(package),
            project: ".".to_string(),
        },
        (package, Some(project)) => CommandKind::Install { package, project },
    }
}

fn looks_like_project_path(value: &str) -> bool {
    value == "." || value.contains('/') || std::path::Path::new(value).is_dir()
}

fn mode_from_args(_dev: bool, prod: bool) -> BuildMode {
    if prod {
        BuildMode::Prod
    } else {
        BuildMode::Dev
    }
}

fn init_command(command: InitCommand) -> CommandKind {
    match command {
        InitCommand::App(path) => CommandKind::Init {
            kind: InitKind::App,
            path: path.path,
        },
        InitCommand::Package(path) => CommandKind::Init {
            kind: InitKind::Package,
            path: path.path,
        },
    }
}

fn xtazy_command(command: XtazyCommand) -> XtazySubcommand {
    match command {
        XtazyCommand::Install(args) => XtazySubcommand::Install {
            version: args.version,
        },
        XtazyCommand::Update => XtazySubcommand::Update,
        XtazyCommand::AutoUpdate(args) => XtazySubcommand::AutoUpdate {
            action: args.action.map(|action| match action {
                AutoUpdateCommand::Off => AutoUpdateAction::Off,
                AutoUpdateCommand::Status => AutoUpdateAction::Status,
            }),
        },
        XtazyCommand::Use(args) => XtazySubcommand::UseVersion {
            version: args.version,
        },
        XtazyCommand::Active => XtazySubcommand::Active,
        XtazyCommand::List => XtazySubcommand::List,
        XtazyCommand::Remove(args) => XtazySubcommand::Remove {
            version: args.version,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parse_version_flag() {
        let cmd = parse_args(&args(&["dealer", "--version"])).expect("version should parse");

        assert_eq!(cmd, CommandKind::Version);
    }

    #[test]
    fn parse_defaults_to_check_for_single_project_argument() {
        let cmd =
            parse_args(&args(&["dealer", "project"])).expect("single project arg should parse");

        assert_eq!(
            cmd,
            CommandKind::Check {
                project: "project".to_string()
            }
        );
    }

    #[test]
    fn parse_check_defaults_to_current_directory() {
        let cmd = parse_args(&args(&["dealer", "check"])).expect("check should parse");

        assert_eq!(
            cmd,
            CommandKind::Check {
                project: ".".to_string()
            }
        );
    }

    #[test]
    fn parse_build_prod_command() {
        let cmd = parse_args(&args(&["dealer", "build", "--prod", "project"]))
            .expect("build command should parse");

        assert_eq!(
            cmd,
            CommandKind::Build {
                project: "project".to_string(),
                mode: BuildMode::Prod
            }
        );
    }

    #[test]
    fn parse_install_package_and_project() {
        let cmd = parse_args(&args(&["dealer", "install", "foo", "project"]))
            .expect("install command should parse");

        assert_eq!(
            cmd,
            CommandKind::Install {
                package: Some("foo".to_string()),
                project: "project".to_string()
            }
        );
    }

    #[test]
    fn parse_xtazy_auto_update_status() {
        let cmd = parse_args(&args(&["dealer", "xtazy", "auto-update", "status"]))
            .expect("auto-update status should parse");

        assert_eq!(
            cmd,
            CommandKind::Xtazy {
                subcommand: XtazySubcommand::AutoUpdate {
                    action: Some(AutoUpdateAction::Status)
                }
            }
        );
    }

    #[test]
    fn parse_self_update() {
        let cmd =
            parse_args(&args(&["dealer", "self", "update"])).expect("self update should parse");

        assert_eq!(cmd, CommandKind::SelfUpdate);
    }
}
