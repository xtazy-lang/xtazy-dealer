# Install And Update

This document defines how `dealer` is installed and how updates are discovered, warned about, or applied.

It covers:

- `dealer` binary install and update
- xtazy build component install and update
- update check caching
- auto-update settings
- warning behavior

Package dependency resolution is a separate project workflow. Package commands use `dealer install`, `dealer update`, `dealer outdated`, and `dealer remove`; they do not update `dealer` itself or xtazy build components.

## Components

There are two independent update domains:

| Domain       | Updated by              | Contains                                               |
|--------------|-------------------------|--------------------------------------------------------|
| `dealer`     | `dealer self update`    | The user-facing `dealer` binary.                       |
| xtazy tools  | `dealer xtazy update`   | xtazy release descriptors, `πko`, `rusttime`, `std`, Rust backend. |

`xtazy 0.1.0` is a release composition, not a single installed folder. The composition file format is defined in [xtazy Parts File](xtazy-parts.md).

The local installed layout is defined in [xtazy Toolchain](toolchain.md).

## First Install

The Linux/macOS install entrypoint is:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.xtazy.dev | sh
```

The installer installs only the `dealer` binary.

Install flow:

| Step | Action |
|------|--------|
| 1 | Download `version.txt` and `targets.tsv` from `https://dealer.xtazy.dev/`. |
| 2 | Detect the machine with `uname -s` and `uname -m`. |
| 3 | Select the matching `unix` row from `targets.tsv`. |
| 4 | Download `dealer-<version>-<suffix>.tar.gz`. |
| 5 | Verify the archive sha256 from `targets.tsv`. |
| 6 | Install `dealer` into the configured binary directory. |
| 7 | Warn if the binary directory is not in `PATH`. |
| 8 | Run `dealer xtazy update` unless the installer was told to skip tool installation. |

The installer must not install `πko`, `rusttime`, `std`, or Rust backend artifacts directly.

Those are installed by `dealer xtazy update`.

## Local State

Global dealer state lives under `~/.dealer/`.

The canonical folder layout is defined in [xtazy Toolchain](toolchain.md).

Settings under `~/.dealer/config/` include:

| Setting                   | Meaning                                                  |
|---------------------------|----------------------------------------------------------|
| `self auto-update`        | Whether `dealer` may update its own binary automatically. |
| `xtazy auto-update`       | Whether xtazy build components may update automatically. |
| `last self update check`  | Last successful self-update metadata check time.         |
| `last xtazy update check` | Last successful xtazy update metadata check time.        |

The exact file format is internal to `dealer`.

## Update Check Cache

Normal commands should not hit the network on every run.

`dealer` keeps update-check timestamps and uses a default check window of one hour.

Dealer self-update checks read:

```text
https://dealer.xtazy.dev/
```

That endpoint returns the latest `dealer` `version.txt`.

xtazy tooling update checks read:

```text
https://dealer.xtazy.dev/xtazy
```

That endpoint returns the latest xtazy `version.txt`, containing only the latest xtazy version number.

Pinned xtazy project versions read:

```text
https://dealer.xtazy.dev/xtazy/v0.1.0/xtazy.parts
https://dealer.xtazy.dev/xtazy/v0.1.0/xtazy.parts.xsigfile
```

The version in the URL is the pinned project version prefixed with `v`. The URL and file contract is defined in [xtazy Parts File](xtazy-parts.md).

| Situation | Behavior |
|-----------|----------|
| Check window still valid | Do not check the network. Continue with local state. |
| Check window expired | Check the relevant update metadata before the command continues. |
| Explicit update command | Ignore the check window and check immediately. |
| Status command | Read local settings/state only. Do not check the network. |

Explicit update commands:

```text
dealer self update
dealer xtazy update
```

Status commands:

```text
dealer self auto-update status
dealer xtazy auto-update status
```

## Self Update

`dealer self update` updates the `dealer` binary.

Flow:

| Step | Action |
|------|--------|
| 1 | Read current installed `dealer` version. |
| 2 | Fetch latest `dealer` version from `https://dealer.xtazy.dev/`. |
| 3 | If the installed version is current, do nothing. |
| 4 | If a newer version exists, download the matching archive. |
| 5 | Verify the archive before use. |
| 6 | Replace the installed `dealer` binary atomically where the platform allows it. |

`dealer self auto-update on` enables automatic self updates.

`dealer self auto-update off` disables automatic self updates.

`dealer self auto-update status` prints the local setting.

## Self Update Warning Rules

When the check window expires and a newer `dealer` exists:

| Setting | Behavior |
|---------|----------|
| `self auto-update off` | Print a warning and continue with the current `dealer`. |
| `self auto-update on`  | Update `dealer` before running the requested command. |

Warning shape:

```text
warning: newer dealer available: <current> -> <latest>
hint: run `dealer self update` or enable `dealer self auto-update on`
```

If automatic self update is enabled and the update succeeds, the requested command runs with the updated `dealer`.

If automatic self update fails, `dealer` must not leave a partial binary behind.

## xtazy Update

`dealer xtazy update` updates the xtazy build components managed by `dealer`.

Flow:

| Step | Action |
|------|--------|
| 1 | Fetch `https://dealer.xtazy.dev/xtazy` to discover the latest xtazy version when the project is unpinned. |
| 2 | Fetch `https://dealer.xtazy.dev/xtazy/v<version>/xtazy.parts` and `.xsigfile` for the selected xtazy version. |
| 3 | Verify `xtazy.parts` before trusting its component versions or hashes. |
| 4 | Resolve component versions for `πko`, `rusttime`, `std`, and Rust backend. |
| 5 | Download missing or outdated components. |
| 6 | Verify every downloaded artifact before use. |
| 7 | Stage complete components without replacing usable local state early. |
| 8 | Mark the release composition usable only after all required components are complete. |

`dealer xtazy auto-update on` enables automatic xtazy component updates.

`dealer xtazy auto-update off` disables automatic xtazy component updates.

`dealer xtazy auto-update status` prints the local setting.

The `xtazy.parts` verification and component hash contract is defined in [xtazy Parts File](xtazy-parts.md).

## Project Version Rules

Projects may pin the xtazy release composition on the root `dealer` line. Project root syntax is defined in [xtazy Projects](project.md).

Pinned project behavior:

| Situation | Behavior |
|-----------|----------|
| Required composition is installed and complete | Use it. |
| Required composition is missing and auto-update is on | Download the exact pinned composition and components before build. |
| Required composition is missing and auto-update is off | Error and tell the user to run `dealer xtazy update`. |
| Newer xtazy release exists | Do not switch the project. The project is pinned. |

Unpinned project behavior:

| Situation | Behavior |
|-----------|----------|
| Local usable composition exists and check window is valid | Use local state. |
| Check window expired and no newer composition exists | Continue with local state. |
| Check window expired, newer composition exists, auto-update is off | Warn and continue with local usable state. |
| Check window expired, newer composition exists, auto-update is on | Update before build/check/run/test. |
| No usable local composition exists | Require `dealer xtazy update`, or run it automatically only when auto-update is on. |

## xtazy Update Warning Rules

When the check window expires and a newer xtazy release composition exists:

| Setting | Behavior |
|---------|----------|
| `xtazy auto-update off` | Print a warning and continue with the current usable local composition. |
| `xtazy auto-update on`  | Update xtazy components before running the project command. |

Warning shape:

```text
warning: newer xtazy tooling available: <current> -> <latest>
hint: run `dealer xtazy update` or enable `dealer xtazy auto-update on`
```

For project commands, automatic xtazy update happens before the project is built or checked.

The command must not build first and update afterward.

## Failure Rules

Update failures must be clean.

| Failure | Behavior |
|---------|----------|
| Metadata fetch fails during explicit update | Error. |
| Metadata fetch fails during cached automatic check | Warn if current local state is usable; otherwise error. |
| Artifact download fails | Error for explicit update; automatic update must not replace current usable state. |
| Signature/hash verification fails | Error and discard the artifact. |
| Component staging is incomplete | Do not mark the release composition usable. |

No partially downloaded or partially verified component may become usable for a build.

Global update and cache mutation must be protected by dealer file locks. Locking details are defined in [xtazy Toolchain](toolchain.md).

## Package Install And Update Boundary

Package commands update project dependencies, not the dealer binary and not xtazy build components.

Package command names are listed in [dealer Commands](commands.md).

Package source syntax, curated registry records, and dependency resolver rules are defined in [Project Dependencies](project-dependencies.md).
