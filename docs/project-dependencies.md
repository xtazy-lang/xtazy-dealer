# Project Dependencies

This page defines how `dealer` resolves dependencies declared in the root `dealer` block.

Project root syntax is defined in [xtazy Projects](project.md). Build/check/run behavior is defined in [Project Build Flow](project-build-flow.md).

## Dependency Forms

External dependencies are declared in the root `dealer` block of `app.x` or `package.x`.

| Shape | Meaning |
|-------|---------|
| `core_utils 1.0.1` | Named package with full numeric compatible requirement from the curated xtazy package index. |
| `core_utils 1.x.x` | Named package with major version wildcard (range matching `1.*.*`). |
| `core_utils 1.2.x` | Named package with minor version wildcard (range matching `1.2.*`). |
| `data_tools "<git-url>" 1.0.3` | Git package matching a numeric version requirement. |
| `data_tools "<git-url>" 1.x.x` | Git package matching a major wildcard tag range. |
| `ui_components "<git-url>" "main"` | Git package pinned to an exact branch or ref. |
| `shared_types "../shared_types"` | Local path package. |

Dependency declarations use numeric requirements, local paths, or git references.

Package CLI commands run against the current project root.

## Resolution

Dependency resolution turns every dependency declaration into one concrete local package root.

`dealer` resolves packages before check, build, run, or test workflow continues.

| Source shape | Resolution |
|--------------|------------|
| `core_utils <version-req>` | Fetch package versions and resolve the highest satisfying version from the curated registry. |
| `data_tools "<git-url>" <version-req>` | Inspect remote git tags, find the highest satisfying tag version, fetch the repository, and check it out. |
| `ui_components "<git-url>" "main"` | Fetch the git repository into dealer cache and check out ref `main`. |
| `shared_types "../shared_types"` | Resolve the path relative to the project root and use it as live local source. |

### Version Ranges and Merging

When multiple transitive dependencies in the dependency graph declare different requirements for the same package, `dealer` merges the requirements to find a compatible range:
- Compatible requirements are intersected (e.g., `1.x.x` and `1.2.x` merge to `1.2.x`).
- The resolver then selects the highest version satisfying the merged requirement from the registry/git tags list.
- If requirements cannot be merged (no overlapping version exists, e.g., `1.1.x` and `1.2.x`), resolution fails.

If a new requirement narrows a version selection during recursive resolution, the graph is re-evaluated, and any stale transitive dependencies (dependencies introduced by versions that were subsequently discarded) are garbage-collected and absent from the final resolved map.

Local path dependencies stay live references, so edits in the local package are visible on the next `dealer check`, `dealer build`, or `dealer run`.

Git and registry dependencies are stored under dealer-managed global cache.

## Package Update Commands

`dealer update [package]` updates dependency declarations in the current project root.

Update behavior:

| Dependency shape | Update behavior |
|------------------|-----------------|
| `core_utils 1.0.1` | Fetch versions from `https://dealer.xtazy.dev/package/<name>` and update to the highest version satisfying the requirement `1.0.1`. |
| `core_utils 1.x.x` | Wildcard requirements are left unchanged by `dealer update`. |
| `data_tools "<git-url>" 1.0.3` | Inspect remote tags and update the requirement to the highest version matching `1.0.3`. |
| `data_tools "<git-url>" 1.x.x` | Wildcard requirements are left unchanged by `dealer update`. |
| `ui_components "<git-url>" "main"` | Not changed by `dealer update`; branch/ref dependencies are not numeric versions. |
| `shared_types "../shared_types"` | Not changed by `dealer update`; local path dependencies are live source references. |

`dealer outdated` uses the same rules to display newer versions satisfying numeric requirements, but does not modify files and skips wildcard requirements.

`dealer remove <package>` removes the matching dependency declaration from the current project root.

## Curated Registry

The curated registry uses package records.

Available versions list:

```text
https://dealer.xtazy.dev/package/<name>
```

This returns a list of all released versions (one per line):

```text
<version> sha256:<hash> <url>
```

Concrete resolved version record:

```text
https://dealer.xtazy.dev/package/<name>/<version>
```

After a requirement is resolved to one concrete package version, this endpoint returns the package record for that selected version:

```text
<version> sha256:<hash> <url>
```

Example:

```text
2.1.0 sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef https://github.com/xtazy-packages/core-utils/archive/refs/tags/v2.1.0.tar.gz
```

`dealer` downloads the URL and accepts the package only when the downloaded content matches the declared SHA-256 hash.

No per-package signature is part of the package registry MVP.

## Resolver Rules

For every resolved package root, `dealer` must verify that `package.x` exists.

The package name declared in `package.x` must match the dependency name used by the dependent project.

`dealer` resolves package dependencies recursively by reading the dependency package root `dealer` block.

The resolver must fail when:

| Failure | Meaning |
|---------|---------|
| Missing package root | A dependency cannot be found or downloaded. |
| Missing `package.x` | The resolved folder is not an xtazy package. |
| Package name mismatch | The dependency name and the package declaration name disagree. |
| Source conflict | The same package name resolves to different roots, incompatible version ranges, or conflicting source types. |
| Cycle | Package dependencies form a cycle. |

The successful resolver result is:

| Field | Meaning |
|-------|---------|
| `entry_file` | Absolute path to the root `app.x` or `package.x`. |
| `project_root` | Absolute path to the project folder. |
| `project_name` | Name from the root `app` or `package` declaration. |
| `resolved_packages` | Map from package name to concrete local package root. |

Example resolver output:

```text
entry_file = /projects/sample_app/app.x
project_root = /projects/sample_app
project_name = sample_app

resolved_packages:
	core_utils = ~/.dealer/cache/packages/core_utils/2.1.0/source
	data_tools = ~/.dealer/cache/git/data_tools/1.0.3/source
	shared_types = /projects/shared_types
```

The resolved package result is passed into the project build flow.
