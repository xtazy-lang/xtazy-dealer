# xtazy Projects

An xtazy project is a folder managed by `dealer`.

This page defines the project folder shape and root declarations.

Command names are listed in [dealer Commands](commands.md). Dependency resolver behavior is defined in [Project Dependencies](project-dependencies.md). Build/check/run behavior is defined in [Project Build Flow](project-build-flow.md).

## Project Shape

A project root contains exactly one root file and one shared source tree.

```text
project/
│
├── app.x | package.x
│   └── project root file
│
├── source files
│
├── .dealer/
│   └── internal project state
│
└── product/
    └── final user-facing output
```

| Path                  | Meaning                                                                      |
|-----------------------|------------------------------------------------------------------------------|
| `app.x`               | Executable app root.                                                         |
| `package.x`           | Reusable package root.                                                       |
| source files           | Project source files consumed by πko.                                        |
| `.dealer/`            | Internal project state owned by `dealer`.                                    |
| `product/`            | Final user-facing output.                                                    |

`.dealer/` is project-local generated state. Global downloads and reusable tooling/package cache live under the dealer state directory, not inside the project.

If both `app.x` and `package.x` exist, the project is ambiguous.

If neither root file exists, the folder is not an xtazy project.

`dealer` discovers the root folder and passes project source to πko.

## App

An app project starts with `app.x`.

```text
app sample_app 1.0.0
	dealer
		core_utils 1.0.1
		shared_types "../shared_types"
```

The app name and app version live directly on the `app` line.

The app name is also the default output artifact name.

The root `dealer` block declares project dependencies for the project.

Projects normally use the latest usable xtazy release composition managed by `dealer`. If a project must pin the xtazy release composition, the pin is written on the `dealer` line.

```text
app sample_app 1.0.0
	dealer xtazy 0.1.0
		core_utils 1.0.1
		shared_types "../shared_types"
```

Dependency declarations are defined in [Project Dependencies](project-dependencies.md).

Pinned and unpinned xtazy tooling update behavior is defined in [Install And Update](install-and-update.md).

## Package

A package project starts with `package.x`.

```text
package sample_core 1.0.0
	dealer
		core_utils 1.0.1
```

The package name and package version live directly on the `package` line.

## Root Dealer Block

External dependencies are declared in the root `dealer` block of `app.x` or `package.x`.

Package CLI commands run against the current project root.

Dependency forms and resolver behavior are defined in [Project Dependencies](project-dependencies.md).
