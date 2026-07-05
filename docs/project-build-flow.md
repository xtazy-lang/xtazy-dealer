# Project Build Flow

This page defines what `dealer` does when checking, building, or running an xtazy project.

Project root syntax is defined in [xtazy Projects](project.md). Dependency resolution is defined in [Project Dependencies](project-dependencies.md).

## Build State

Project-local generated state lives under `.dealer/`.

Final user-facing output lives under `product/`.

```text
project/
│
├── .dealer/
│   ├── rust/
│   │   └── generated Rust project and Rust backend build state
│   │
│   └── xtazy/
│       └── dependency graph, build fingerprints, and xtazy build state
│
└── product/
    └── final user-facing output
```

Global downloads and reusable tooling/package cache live under the dealer state directory, not inside the project.

## Check Flow

`dealer check` validates the project without producing final output.

Flow:

| Step | Action |
|------|--------|
| 1 | Discover the project root. |
| 2 | Resolve dependencies into local package roots. |
| 3 | Write a πko check request under `.dealer/xtazy/piko/request/`. |
| 4 | Run the selected πko executable with that request. |
| 5 | Read the πko result from `.dealer/xtazy/piko/result/`. |
| 6 | Print πko diagnostics. |
| 7 | Exit successfully only when πko reports no errors. |

`dealer check` does not run the Rust backend and does not write `product/`.

## Build Flow

`dealer build` produces the final artifact.

Flow:

| Step | Action |
|------|--------|
| 1 | Discover the project root. |
| 2 | Resolve dependencies into local package roots. |
| 3 | Prepare `.dealer/rust/` and `.dealer/xtazy/`. |
| 4 | Write a πko build request under `.dealer/xtazy/piko/request/`. |
| 5 | Run the selected πko executable with that request. |
| 6 | Read the πko result from `.dealer/xtazy/piko/result/`. |
| 7 | Stop if πko reports errors. |
| 8 | Run the selected Rust backend inside `.dealer/rust/`. |
| 9 | Copy the final artifact into `product/`. |

πko writes generated Rust only.

`dealer` runs the Rust backend and owns `product/`.

## Run Flow

`dealer run` builds and runs an app project.

Flow:

| Step | Action |
|------|--------|
| 1 | Run the build flow. |
| 2 | Execute the generated app artifact from `product/`. |
| 3 | Return the app exit status. |

Package roots are not runnable.

## Test Flow

`dealer test` runs project tests through πko.

Flow:

| Step | Action |
|------|--------|
| 1 | Discover the project root. |
| 2 | Resolve dependencies into local package roots. |
| 3 | Write a πko test request under `.dealer/xtazy/piko/request/`. |
| 4 | Run the selected πko executable with that request. |
| 5 | Read the πko result from `.dealer/xtazy/piko/result/`. |
| 6 | Print πko diagnostics and test output. |
| 7 | Exit successfully only when πko reports success. |

`dealer` does not interpret test syntax.

## Format Flow

`dealer fmt` formats only the current project source tree.

It does not format registry packages, git packages, cached packages, or local path dependencies.

To format a local dependency, run `dealer fmt` from that dependency project root.

Flow:

| Step | Action |
|------|--------|
| 1 | Discover the current project root. |
| 2 | Write a πko format request under `.dealer/xtazy/piko/request/`. |
| 3 | Run the selected πko executable with that request. |
| 4 | Read the πko result from `.dealer/xtazy/piko/result/`. |
| 5 | Print πko diagnostics. |

`dealer fmt --check` uses the same flow with mode `fmt-check`.

`dealer` does not parse or format Xtazy source itself.

## Dealer To πko Tool Protocol

πko is an independent toolchain executable selected by the active xtazy release composition.

`dealer` does not link πko as a Cargo dependency.

`dealer` invokes πko through a tool-to-tool protocol.

The protocol is not a user-facing CLI.

## Request Directory

Before invoking πko, `dealer` writes:

```text
<project_root>/.dealer/xtazy/piko/request/
```

The request directory contains UTF-8 text files:

```text
request/
├── mode
├── entry_file
├── project_root
├── project_name
├── color
├── resolved_packages.tsv
├── rust_output_dir
├── generated_package_name
└── rusttime_path
```

Required files:

| Field | Meaning |
|-------|---------|
| `mode` | `check`, `build`, `test`, `fmt`, or `fmt-check`. |
| `entry_file` | Absolute path to the root `app.x` or `package.x`. |
| `project_root` | Absolute path to the project folder. |
| `project_name` | Name from the root `app` or `package` declaration. |
| `color` | `auto`, `always`, or `never`. |
| `resolved_packages.tsv` | Resolved package map for `check`, `build`, and `test`. Empty or omitted for `fmt` and `fmt-check`. |

Build-only files:

| Field | Meaning |
|-------|---------|
| `rust_output_dir` | Absolute path to `<project_root>/.dealer/rust/`. |
| `generated_package_name` | Rust package name, defaulting to `project_name`. |
| `rusttime_path` | Absolute path to the Rust-facing support crate selected by the xtazy release composition. |

Single-value files contain exactly one value followed by an optional final newline.

`dealer` chooses the `color` value before invoking πko.

πko writes `diagnostics` in the requested color mode.

`resolved_packages.tsv` uses one package per line:

```text
<package-name>\t<absolute-package-root>
```

Example:

```text
core_utils	/Users/example/.dealer/cache/packages/core_utils/2.1.0/source
data_tools	/Users/example/.dealer/cache/git/data_tools/1.0.3/source
shared_types	/Users/example/projects/shared_types
```

Every path written by `dealer` must be absolute.

Package names and paths written into `resolved_packages.tsv` must not contain tabs or newlines.

Package registry, git, and cache resolution must already be complete before this request is written.

## Invocation

`dealer` invokes the selected πko executable with explicit request and result directories:

```text
piko --request <project_root>/.dealer/xtazy/piko/request --result <project_root>/.dealer/xtazy/piko/result
```

`dealer` must remove any previous `result/` directory before invoking πko.

`dealer` must treat failure to start πko as a dealer/toolchain error.

`dealer` waits for the πko process to exit before reading the result directory.

If πko exits with a non-zero process status and still writes a readable result directory, `dealer` uses the result directory as the protocol result.

If πko exits with a non-zero process status and does not write a readable result directory, `dealer` reports a toolchain error and may include stdout/stderr as fallback debugging text.

`dealer` must not parse stdout or stderr as the normal protocol.

The normal protocol result is the result directory.

## Result Directory

After πko exits, `dealer` reads:

```text
<project_root>/.dealer/xtazy/piko/result/
```

Expected files:

```text
result/
├── status
└── diagnostics
```

`status` contains one of:

| Status | Dealer behavior |
|--------|-----------------|
| `ok` | Continue workflow. |
| `warning` | Print diagnostics and continue workflow. |
| `error` | Print diagnostics and stop workflow. |

`diagnostics` is human-readable text printed by `dealer`.

If πko exits without writing a readable `status` file, `dealer` treats that as a toolchain error.

For `check`, no Rust output is expected.

For `build`, `ok` or `warning` means `rust_output_dir` must contain a generated Rust project ready for the selected Rust backend.

## Check Request

For `dealer check`, request files are:

```text
mode = check
entry_file
project_root
project_name
color
resolved_packages.tsv
```

`dealer check` does not provide `rust_output_dir`, `generated_package_name`, or `rusttime_path`.

`dealer check` must not run the Rust backend.

`dealer check` must not write `product/`.

## Test Request

For `dealer test`, request files are:

```text
mode = test
entry_file
project_root
project_name
color
resolved_packages.tsv
```

`dealer test` does not run the Rust backend.

`dealer test` does not write `product/`.

## Format Request

For `dealer fmt`, request files are:

```text
mode = fmt
entry_file
project_root
project_name
color
```

For `dealer fmt --check`, `mode` is:

```text
fmt-check
```

`dealer fmt` formats only files in the current project root.

`dealer fmt --check` must not write source files.

## Build Request

For `dealer build`, request files are:

```text
mode = build
entry_file
project_root
project_name
color
resolved_packages.tsv
rust_output_dir
generated_package_name
rusttime_path
```

`rust_output_dir` is:

```text
<project_root>/.dealer/rust/
```

After a successful πko result, `dealer` runs the selected Rust backend inside `rust_output_dir`.

After a successful Rust backend build, `dealer` copies the final artifact into `product/`.
