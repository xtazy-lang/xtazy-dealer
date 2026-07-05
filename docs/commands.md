# dealer Commands

`dealer` is the user-facing command for xtazy projects, packages, builds, and local tooling.

This page lists command names and intent only.

Behavior details live in:

| Area | Canonical page |
|------|----------------|
| Project layout and root declarations | [xtazy Projects](project.md) |
| Dependency syntax and resolution | [Project Dependencies](project-dependencies.md) |
| Check/build/run flow | [Project Build Flow](project-build-flow.md) |
| Install, update, auto-update, and warning flow | [Install And Update](install-and-update.md) |
| Local toolchain and cache layout | [xtazy Toolchain](toolchain.md) |

## dealer self

Commands for the `dealer` binary itself.

| Command                                     | Purpose                                                        |
|---------------------------------------------|----------------------------------------------------------------|
| `dealer self update`                        | Update the installed `dealer` binary.                          |
| `dealer self auto-update <on\|off\|status>` | Enable, disable, or inspect automatic `dealer` binary updates. |

## dealer xtazy

Commands for the local xtazy tooling used by `dealer`.

| Command                                      | Purpose                                                      |
|----------------------------------------------|--------------------------------------------------------------|
| `dealer xtazy update`                        | Update xtazy tooling used by `dealer`.                       |
| `dealer xtazy auto-update <on\|off\|status>` | Enable, disable, or inspect automatic xtazy tooling updates. |
| `dealer xtazy doctor`                        | Inspect the local dealer, project, and xtazy tooling setup.  |

## Project Workflow

Commands for the normal project loop.

| Command                                  | Purpose                                                              |
|------------------------------------------|----------------------------------------------------------------------|
| `dealer build [--dev\|--prod] [project]` | Build a project, keep internal state in `.dealer/`, and write final output to `product/`. |
| `dealer run [--dev\|--prod] [project]`   | Build and run an app project; package roots are not runnable.        |
| `dealer test [project]`                  | Run project tests.                                                   |
| `dealer check [project]`                 | Check an xtazy project without producing final output.               |
| `dealer clean [project]`                 | Remove `.dealer/` and `product/`; source files are not touched.      |

## Project Creation

Commands for creating new project folders.

| Command                      | Purpose                   |
|------------------------------|---------------------------|
| `dealer init app [path]`     | Create an app project.    |
| `dealer init package [path]` | Create a package project. |

## Formatting

Commands for source formatting.

| Command                        | Purpose                                   |
|--------------------------------|-------------------------------------------|
| `dealer fmt [project]`         | Format an xtazy project.                  |
| `dealer fmt --check [project]` | Check formatting without writing changes. |

## Packages

Commands for package dependencies of the current project.

Package commands intentionally use the current working directory as the project root. They do not accept a positional project path, so dependency sources can be parsed without ambiguity.

| Command                            | Purpose                                                                  |
|------------------------------------|--------------------------------------------------------------------------|
| `dealer install`                   | Install packages already declared by the current project.                |
| `dealer install <name>`            | Add and install a named curated package.                                 |
| `dealer install <path>`            | Add and install a local path package.                                    |
| `dealer install <git-url>`         | Add and install a git package source.                                    |
| `dealer update [package]`          | Update all project packages, or only the selected package when provided. |
| `dealer outdated`                  | Show declared packages with newer available versions.                    |
| `dealer remove <package>`          | Remove a package from the current project.                               |

## dealer cache

Commands for dealer-managed global cache state.

| Command              | Purpose                                                                 |
|----------------------|-------------------------------------------------------------------------|
| `dealer cache clean` | Remove dealer-managed global cache data outside a single project folder. |
