# dealer Release

This page documents the release flow for the `dealer` binary and its GitHub Release assets.

Runtime install and update behavior is defined in [Install And Update](install-and-update.md).

## Trigger

Dealer releases are started manually from GitHub Actions.

| Rule                    | Meaning                                                |
|-------------------------|--------------------------------------------------------|
| Trigger                 | `workflow_dispatch`                                    |
| Version source          | `Cargo.toml`                                           |
| Release tag             | `v<version>`                                           |
| Tag-triggered release   | Not used                                               |
| Manual version input    | Not used                                               |

The release version is read from `Cargo.toml`. The workflow does not accept a separate version input.

## Release Guard

The first job is `preflight`.

It uses:

```text
xtazy-lang/ci-cd-helpers-xtazy/.github/actions/release-guard@v0.1.0
```

Inputs:

| Input        | Value        |
|--------------|--------------|
| `manifest`   | `Cargo.toml` |
| `tag-prefix` | `v`          |

The guard must fail before any release build when:

| Check                  | Rule                                                 |
|------------------------|------------------------------------------------------|
| Manifest version       | Version must be readable from `Cargo.toml`.          |
| Semver order           | Version must be greater than the latest release tag. |
| Existing release tag   | `v<version>` must not already exist.                 |

The guard outputs:

| Output    | Meaning            |
|-----------|--------------------|
| `version` | Release version.   |
| `tag`     | Release tag name.  |

## Test Gate

The `test` job runs before release artifacts are built.

Commands:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --quiet
```

If this job fails, no dealer release artifacts are built.

## Build Artifacts

The `build` job uses the reusable Rust release build workflow:

```text
xtazy-lang/ci-cd-helpers-xtazy/.github/workflows/rust-release-build.yml@v0.1.0
```

Inputs:

| Input               | Value                               |
|---------------------|-------------------------------------|
| `binary-name`       | `dealer`                            |
| `package-name`      | `xtazy-dealer`                      |
| `artifact-prefix`   | `dealer`                            |
| `version-source`    | `cargo`                             |
| `package-format`    | `tar.gz`                            |
| `result-files-file` | `.github/release/result_files.json` |

Release archive naming:

```text
dealer-<version>-<suffix>.tar.gz
```

The suffix comes from the shared Rust build matrix.

The archive must unpack to:

```text
dealer/dealer
```

for Linux and macOS archives.

## Result Files

The build job creates release result files from:

```text
.github/release/result_files.json
```

Current result file:

| File          | Source template                             |
|---------------|---------------------------------------------|
| `targets.tsv` | `.github/release/templates/targets.tsv.xtpl` |

`targets.tsv` rows have this shape:

```text
family<TAB>lookup_os_lookup_arch<TAB>suffix<TAB>sha256
```

The installer uses `targets.tsv` to select the right dealer archive and verify its sha256.

## Documentation

The `docs` job uses:

```text
xtazy-lang/ci-cd-helpers-xtazy/.github/workflows/docs-site.yml@v0.1.0
```

Inputs:

| Input              | Value                   |
|--------------------|-------------------------|
| `crate-name`       | `xtazy-dealer`          |
| `out-dir`          | `docs-out`              |
| `include-rust-api` | `true`                  |
| `cname`            | `docs.dealer.xtazy.dev` |

Documentation is published to the docs site.

Documentation is not uploaded as a GitHub Release archive.

## Signing

Only `targets.tsv` is signed in the dealer release flow.

The workflow uses:

```text
xtazy-lang/ci-cd-helpers-xtazy/.github/actions/sign-files@v0.1.0
```

Inputs:

| Input          | Value                                   |
|----------------|-----------------------------------------|
| `files`        | `release-upload/targets.tsv`            |
| `private`      | `.release-signing/xtazy_dealer.private` |
| `delegation`   | `.release-signing/xtazy_dealer.xsig`    |
| `xsig-version` | `v0.1.0`                                |

Environment:

| Variable        | Source                 |
|-----------------|------------------------|
| `XSIG_PASSWORD` | GitHub release secret. |

Output:

```text
targets.tsv.xsigfile
```

Dealer archives are not signed individually in this flow. Their sha256 values are stored inside the signed `targets.tsv`.

## Publish

The `publish` job waits for:

| Required job | Meaning                                             |
|--------------|-----------------------------------------------------|
| `preflight`  | Version and release-tag guard has passed.           |
| `build`      | Dealer archives and result files have been created. |
| `docs`       | Dealer documentation has been generated/deployed.   |

It stages release files under:

```text
release-upload/
```

The staged release assets are:

| Asset                              | Purpose                                |
|------------------------------------|----------------------------------------|
| `version.txt`                      | Release version.                       |
| `targets.tsv`                      | Installer target lookup and checksums. |
| `targets.tsv.xsigfile`             | Signature file for `targets.tsv`.      |
| `dealer-<version>-<suffix>.tar.gz` | Dealer binary archive.                 |

The GitHub Release is created with:

```sh
gh release create "$RELEASE_TAG" release-upload/* \
	--target "$GITHUB_SHA" \
	--title "$RELEASE_TAG" \
	--notes "Release $RELEASE_TAG"
```

## Release Keys

Release signing material lives in:

```text
.release-signing/
	xtazy_dealer.private
	xtazy_dealer.xsig
```

The master public trust anchor for runtime verification lives in:

```text
src/trust/master.public
```

The encrypted private key is committed so the same release identity can work across CI providers. The password is not committed; CI receives it through `XSIG_PASSWORD`.
