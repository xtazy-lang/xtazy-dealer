# dealer
***package manager for the xtazy programming language***

`dealer` is the command-line tool for working with xtazy projects.

It is responsible for the everyday project workflow:

- create an xtazy app or package
- check source code
- build the project
- run app projects
- keep generated build files out of the source tree
- place final user-facing output in `product/`
- manage xtazy build components and updates behind the scenes

Generated Rust API docs: [docs.dealer.xtazy.dev](https://docs.dealer.xtazy.dev)

## Documentation

Start here:

| Page | What it covers |
|------|----------------|
| [Commands](docs/commands.md) | User-facing `dealer` commands for projects, packages, cache, self-update, and xtazy tooling. |
| [Projects](docs/project.md) | Project layout, root declarations, project name/version, and the root `dealer` block. |
| [Project Dependencies](docs/project-dependencies.md) | Dependency declaration forms, local/git/registry resolution, and resolver output. |
| [Project Build Flow](docs/project-build-flow.md) | Check/build/run flow, πko handoff, generated Rust, Rust backend, and `product/`. |
| [Install And Update](docs/install-and-update.md) | Installer flow, update checks, auto-update behavior, warnings, and failure rules. |
| [Toolchain](docs/toolchain.md) | Local dealer state, independent xtazy components, Rust backend separation, and cache layout. |
| [xtazy Parts](docs/xtazy-parts.md) | The signed `xtazy.parts` release composition format used by `dealer xtazy update`. |
| [dealer Release](docs/release-dealer.md) | Dealer release workflow, artifacts, `targets.tsv`, signing, and publishing. |

The docs in this repository describe the intended dealer contract for the current 0.1.0 direction.

## License

This project is dual-licensed. You can use it under:

* **[MIT License](LICENSE-MIT)**
* **[ARAF License](LICENSE-ARAF)**
