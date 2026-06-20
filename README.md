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
- manage the selected xtazy toolchain behind the scenes

Generated Rust API docs: [docs.dealer.xtazy.dev](https://docs.dealer.xtazy.dev)

## Basic Usage

Create a new app:

```text
dealer init app [path]
```

Create a new package:

```text
dealer init package [path]
```

Check and build a project:

```text
dealer check [project]
dealer build [project]
```

Run an app project:

```text
dealer run [project]
```

Build profiles:

```text
dealer build --dev [project]
dealer build --prod [project]
dealer run --dev [project]
dealer run --prod [project]
```

Clean generated state:

```text
dealer clean [project]
```

## Project Layout

An xtazy project is a folder with exactly one root file:

```text
app.x
package.x
```

`app.x` is the root of an executable app.

`package.x` is the root of a reusable package.

During a build, `dealer` keeps generated and final files separated:

```text
project/
│
├── app.x | package.x
│   └── project root file
│
├── .dealer/
│   ├── rust/
│   │   └── generated Rust project
│   │
│   ├── metadata/
│   │   └── build snapshots, selected toolchain, output path
│   │
│   └── logs/
│       └── workflow logs
│
└── product/
    └── final user-facing output
```

Meaning:

- `.dealer/` is internal working state owned by `dealer`.
- `.dealer/rust/` contains the generated Rust project used for backend compilation.
- `.dealer/metadata/` stores small build snapshots such as the selected toolchain and output path.
- `.dealer/logs/` is reserved for workflow logs.
- `product/` contains the final output the user should run, ship, or inspect.

xtazy source files stay clean; generated Rust lives in `.dealer/rust/`.

## Toolchains

xtazy uses Rust as its backend target, but normal xtazy projects are driven through `dealer`.

The selected xtazy toolchain provides:

- πko (`piko`), the xtazy compiler backend
- `rusttime`, the Rust-facing support surface used by generated Rust
- `std`, the future xtazy standard library surface

The Rust compiler backend is tracked separately from the xtazy toolchain, so a Rust backend update does not have to be the same thing as an xtazy language/toolchain update.

Local toolchain state lives under:

```text
~/.dealer/
```

Typical state includes:

```text
~/.dealer/config/active-toolchain
~/.dealer/config/auto-update
~/.dealer/config/active-rust-backend
~/.dealer/xtazy/<version>/
~/.dealer/rust/<backend>/
```

Toolchain commands:

```text
dealer xtazy active
dealer xtazy list
dealer xtazy use <version>
dealer xtazy remove <version>
dealer xtazy auto-update
dealer xtazy auto-update off
dealer xtazy auto-update status
```

## Command Reference

```text
dealer --version
dealer -V
dealer check [project]
dealer build [--dev|--prod] [project]
dealer run [--dev|--prod] [project]
dealer clean [project]
dealer init app [path]
dealer init package [path]
dealer doctor
dealer xtazy active
dealer xtazy list
dealer xtazy use <version>
dealer xtazy remove <version>
dealer xtazy auto-update
dealer xtazy auto-update off
dealer xtazy auto-update status
```

Planned command surface:

```text
dealer fmt [--check] [project]
dealer install [project]
dealer install <package> [project]
dealer xtazy install [version]
dealer xtazy update
dealer self update
```

## License

This project is dual-licensed. You can use it under:

* **[MIT License](LICENSE-MIT)**
* **[ARAF License](LICENSE-ARAF)**
