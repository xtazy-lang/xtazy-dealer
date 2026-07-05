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
    },
    Update {
        package: Option<String>,
    },
    Outdated,
    Remove {
        package: String,
    },
    CacheClean,
    Build {
        project: String,
        mode: BuildMode,
    },
    Run {
        project: String,
        mode: BuildMode,
    },
    Test {
        project: String,
    },
    Clean {
        project: String,
    },
    Init {
        kind: InitKind,
        path: Option<String>,
    },
    Tooling {
        subcommand: ToolingSubcommand,
    },
    SelfUpdate,
    SelfAutoUpdate {
        action: AutoUpdateAction,
    },
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
pub(crate) enum ToolingSubcommand {
    Update,
    AutoUpdate { action: AutoUpdateAction },
    Doctor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AutoUpdateAction {
    On,
    Off,
    Status,
}

#[derive(Parser, Debug)]
#[command(
    name = "dealer",
    about = "xtazy project orchestrator",
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
    Update(UpdateArgs),
    Outdated,
    Remove(RemoveArgs),
    Cache(CacheArgs),
    Build(BuildArgs),
    Run(BuildArgs),
    Test(ProjectArg),
    Clean(ProjectArg),
    Init(InitArgs),
    #[command(name = "xtazy")]
    Tooling(ToolingArgs),
    #[command(name = "self")]
    SelfCommand(SelfArgs),
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
}

#[derive(Args, Debug)]
struct UpdateArgs {
    package: Option<String>,
}

#[derive(Args, Debug)]
struct RemoveArgs {
    package: String,
}

#[derive(Args, Debug)]
struct CacheArgs {
    #[command(subcommand)]
    command: CacheSubcommand,
}

#[derive(Subcommand, Debug)]
enum CacheSubcommand {
    Clean,
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
struct ToolingArgs {
    #[command(subcommand)]
    command: ToolingCommand,
}

#[derive(Subcommand, Debug)]
enum ToolingCommand {
    Update,
    #[command(name = "auto-update")]
    AutoUpdate(AutoUpdateArgs),
    Doctor,
}

#[derive(Args, Debug)]
struct AutoUpdateArgs {
    #[command(subcommand)]
    action: AutoUpdateCommand,
}

#[derive(Subcommand, Debug)]
enum AutoUpdateCommand {
    On,
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
    #[command(name = "auto-update")]
    AutoUpdate(AutoUpdateArgs),
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
            | "update"
            | "outdated"
            | "remove"
            | "cache"
            | "build"
            | "run"
            | "test"
            | "clean"
            | "init"
            | "xtazy"
            | "self"
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
        DealerCommand::Install(args) => CommandKind::Install {
            package: args.package,
        },
        DealerCommand::Update(args) => CommandKind::Update {
            package: args.package,
        },
        DealerCommand::Outdated => CommandKind::Outdated,
        DealerCommand::Remove(args) => CommandKind::Remove {
            package: args.package,
        },
        DealerCommand::Cache(args) => match args.command {
            CacheSubcommand::Clean => CommandKind::CacheClean,
        },
        DealerCommand::Build(args) => CommandKind::Build {
            project: args.project,
            mode: mode_from_args(args.dev, args.prod),
        },
        DealerCommand::Run(args) => CommandKind::Run {
            project: args.project,
            mode: mode_from_args(args.dev, args.prod),
        },
        DealerCommand::Test(args) => CommandKind::Test {
            project: args.project,
        },
        DealerCommand::Clean(args) => CommandKind::Clean {
            project: args.project,
        },
        DealerCommand::Init(args) => init_command(args.kind),
        DealerCommand::Tooling(args) => CommandKind::Tooling {
            subcommand: tooling_command(args.command),
        },
        DealerCommand::SelfCommand(args) => match args.command {
            SelfCommand::Update => CommandKind::SelfUpdate,
            SelfCommand::AutoUpdate(args) => CommandKind::SelfAutoUpdate {
                action: auto_update_action(args.action),
            },
        },
    }
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

fn tooling_command(command: ToolingCommand) -> ToolingSubcommand {
    match command {
        ToolingCommand::Update => ToolingSubcommand::Update,
        ToolingCommand::AutoUpdate(args) => ToolingSubcommand::AutoUpdate {
            action: auto_update_action(args.action),
        },
        ToolingCommand::Doctor => ToolingSubcommand::Doctor,
    }
}

fn auto_update_action(command: AutoUpdateCommand) -> AutoUpdateAction {
    match command {
        AutoUpdateCommand::On => AutoUpdateAction::On,
        AutoUpdateCommand::Off => AutoUpdateAction::Off,
        AutoUpdateCommand::Status => AutoUpdateAction::Status,
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
    fn parse_install_package() {
        let cmd =
            parse_args(&args(&["dealer", "install", "foo"])).expect("install command should parse");

        assert_eq!(
            cmd,
            CommandKind::Install {
                package: Some("foo".to_string()),
            }
        );
    }

    #[test]
    fn parse_xtazy_auto_update_status() {
        let cmd = parse_args(&args(&["dealer", "xtazy", "auto-update", "status"]))
            .expect("auto-update status should parse");

        assert_eq!(
            cmd,
            CommandKind::Tooling {
                subcommand: ToolingSubcommand::AutoUpdate {
                    action: AutoUpdateAction::Status
                }
            }
        );
    }

    #[test]
    fn parse_self_auto_update_on() {
        let cmd = parse_args(&args(&["dealer", "self", "auto-update", "on"]))
            .expect("self auto-update on should parse");

        assert_eq!(
            cmd,
            CommandKind::SelfAutoUpdate {
                action: AutoUpdateAction::On
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
