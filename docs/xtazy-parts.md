# xtazy Parts File

The `xtazy.parts` file defines one xtazy release composition.

It is release infrastructure read by `dealer xtazy update`.

It is not a user project manifest.

Local installation layout is defined in [xtazy Toolchain](toolchain.md). Update timing and failure behavior are defined in [Install And Update](install-and-update.md).

## URLs

Latest xtazy version number:

```text
https://dealer.xtazy.dev/xtazy
```

That endpoint returns the latest xtazy `version.txt` content, for example:

```text
0.1.0
```

Specific xtazy release composition:

```text
https://dealer.xtazy.dev/xtazy/v0.1.0/xtazy.parts
https://dealer.xtazy.dev/xtazy/v0.1.0/xtazy.parts.xsigfile
```

## Format

The file is fixed-line, labeled text.

```text
xtazy 0.1.0
piko 0.1.1 sha256:<piko-targets-tsv-sha256>
rusttime 0.1.2 sha256:<rusttime-source-archive-sha256>
std 0.1.8 sha256:<std-source-archive-sha256>
rust 1.88.0 sha256:<rust-channel-manifest-sha256>
```

Rules:

- The file must have exactly 5 non-empty lines.
- UTF-8 text.
- LF line endings.
- No comments.
- No extra fields.
- Lines must appear in this exact order: `xtazy`, `piko`, `rusttime`, `std`, `rust`.
- Version values must be valid semver without `v`.
- Hash values must use `sha256:<64 lowercase hex characters>`.
- The `xtazy` line has no hash because the `xtazy.parts` file itself is the signed release composition.

`xtazy.parts.xsigfile` signs `xtazy.parts`.

`dealer` must verify the signature before trusting component versions or hashes inside `xtazy.parts`.

## Example

```text
xtazy 0.1.0
piko 0.1.1 sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
rusttime 0.1.2 sha256:1111111111111111111111111111111111111111111111111111111111111111
std 0.1.8 sha256:2222222222222222222222222222222222222222222222222222222222222222
rust 1.88.0 sha256:3333333333333333333333333333333333333333333333333333333333333333
```

This means:

| Component | Version | Hash Meaning |
|-----------|---------|--------------|
| xtazy composition | `0.1.0` | The `xtazy.parts` file is the composition authority. |
| `πko` | `0.1.1` | SHA-256 of `piko` `targets.tsv`. |
| `rusttime` | `0.1.2` | SHA-256 of the `rusttime` source archive. |
| `std` | `0.1.8` | SHA-256 of the `std` source archive. |
| Rust | `1.88.0` | SHA-256 of the official Rust channel manifest. |

## Rust Verification

Rust is not hosted by xtazy.

For Rust `1.88.0`, `dealer` downloads the official Rust channel manifest:

```text
https://static.rust-lang.org/dist/channel-rust-1.88.0.toml
```

`dealer` verifies that manifest against the `rust` line hash in `xtazy.parts`.

After the manifest is verified, `dealer` reads the official Rust package URL and hash for the current host target from that manifest.

The Rust archive is accepted only if its hash matches the verified manifest.

## Component Downloads

The xtazy release router provides stable component roots:

```text
https://dealer.xtazy.dev/piko
https://dealer.xtazy.dev/rusttime
https://dealer.xtazy.dev/std
```

Those root endpoints return each component latest `version.txt`.

For an xtazy composition build, `dealer` must not use component latest versions.

It must use the exact versions from `xtazy.parts`:

```text
https://dealer.xtazy.dev/piko/v0.1.1/targets.tsv
https://dealer.xtazy.dev/rusttime/v0.1.2/rusttime-0.1.2.tar.gz
https://dealer.xtazy.dev/std/v0.1.8/std-0.1.8.tar.gz
```

The downloaded files are accepted only if their SHA-256 values match the hashes pinned in `xtazy.parts`.
