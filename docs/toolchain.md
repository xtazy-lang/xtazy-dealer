# xtazy Toolchain

`dealer` manages the local build components needed to build xtazy projects.

`xtazy 0.1.0` is not one physical runtime folder. It is a release composition that tells `dealer` which component versions belong together.

The real installed components are independent:

- `πko`
- `rusttime`
- `std`
- Rust backend

This lets xtazy releases move independently from Rust backend releases and lets multiple xtazy releases share the same installed component versions when that is valid.

## State Layout

Global dealer state lives under `~/.dealer/`.

```text
~/.dealer/
│
├── config/
│   └── dealer settings
│
├── xtazy/
│   └── versions/
│       └── <xtazy-version>/
│           └── release descriptor
│
├── piko/
│   └── <piko-version>/
│       └── piko
│
├── rusttime/
│   └── <rusttime-version>/
│
├── std/
│   └── <std-version>/
│
├── rust/
│   └── <rust-backend-id>/
│       ├── bin/
│       │   ├── rustc
│       │   └── cargo
│       └── lib/
│
└── cache/
```

| Path                                | Meaning                                                 |
|-------------------------------------|---------------------------------------------------------|
| `~/.dealer/config/`                 | Dealer settings such as auto-update state.              |
| `~/.dealer/xtazy/versions/<ver>/`   | xtazy release composition descriptor.                   |
| `~/.dealer/piko/<ver>/`             | Installed `πko` compiler executable version.            |
| `~/.dealer/rusttime/<ver>/`         | Installed Rust-facing support surface version.          |
| `~/.dealer/std/<ver>/`              | Installed xtazy standard library surface version.       |
| `~/.dealer/rust/<backend>/`         | Installed Rust backend used to compile generated Rust.  |
| `~/.dealer/cache/`                  | Global download and verification cache.                 |

## xtazy Release Composition

An xtazy release descriptor maps an xtazy release version to concrete component versions.

The canonical file format, signature rule, URL shape, and component hash meaning are defined in [xtazy Parts File](xtazy-parts.md).

`dealer` can use an xtazy release only when every referenced component is installed and verified. Incomplete component sets must not be used for builds.

## Project Version Selection

Version selection belongs to the project root when the project needs pinning.

Project root syntax is defined in [xtazy Projects](project.md).

Pinned and unpinned update behavior is defined in [Install And Update](install-and-update.md).

## Dealer Settings

Dealer settings are stored under `~/.dealer/config/`.

This is internal dealer state, not a user project manifest.

Settings include:

| Setting                  | Meaning                                                  |
|--------------------------|----------------------------------------------------------|
| `self auto-update`       | Whether `dealer` may update itself automatically.        |
| `xtazy auto-update`      | Whether xtazy build components may update automatically. |
| last update check times  | Cache timestamps for update checks.                      |

The exact on-disk format is an implementation detail of `dealer`.

Project behavior belongs in `app.x` or `package.x`, not in JSON/TOML/YAML project manifests.

## Rust Backend

Rust is a backend target for xtazy, not a prerequisite the user should manage manually.

The Rust backend is stored separately from xtazy components so Rust backend updates and xtazy component updates can move independently.

Example:

```text
~/.dealer/rust/default-2026-07/
	bin/
		rustc
		cargo
	lib/
```

`dealer` uses the Rust backend referenced by the xtazy release composition used for that build to compile generated Rust from the project-local `.dealer/rust/` folder.

## Update

`dealer xtazy update` updates dealer-managed xtazy build components.

`dealer xtazy auto-update <on|off|status>` controls automatic xtazy component updates.

The full install/update decision flow is defined in [Install And Update](install-and-update.md).

## Cache

`~/.dealer/cache/` is global dealer cache.

It may contain downloaded metadata, downloaded artifacts, verified temporary files, and reusable package/tooling data.

Project-local generated state is not stored here. Project-local state lives under the project `.dealer/` folder.

`dealer cache clean` removes global cache data.

It does not remove installed xtazy release compositions, installed `πko`, installed `rusttime`, installed `std`, or installed Rust backends.

## Locking

Operations that mutate global dealer state must use file locking under `~/.dealer/`.

This includes:

| Operation | Protected state |
|-----------|-----------------|
| xtazy tooling update | `~/.dealer/xtazy/`, `~/.dealer/piko/`, `~/.dealer/rusttime/`, `~/.dealer/std/`, `~/.dealer/rust/` |
| package download or extraction | `~/.dealer/cache/` |
| cache clean | `~/.dealer/cache/` |

Locking must prevent two concurrent `dealer` processes from extracting, replacing, or deleting the same global cache/tooling path at the same time.

`dealer clean [project]` removes project-local generated state and `product/`.
