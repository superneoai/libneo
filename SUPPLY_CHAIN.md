# Supply-chain review

Assessment date: 2026-08-21. Advisory exceptions must be reviewed by 2026-11-21.

## Accepted advisories

### RUSTSEC-2024-0436 (`paste` 1.0.15)

RUSTSEC classifies `paste` as unmaintained, not vulnerable, and lists no patched
versions. It enters the macOS graph through both `metal -> paste` and
`image -> exr -> pulp -> paste`, below the pinned GPUI packages. It runs as a
procedural macro while dependencies compile; no `paste` code remains callable at
runtime. The generated code is used by the Metal and image stacks, so the
compile-time dependency is reachable.

The database suggests replacement crates, but they require source changes in the
transitive dependants and are not lockfile-compatible upgrades. The current Zed
main revision, `91bf967e279fba3b326c096aeb66053cb2373547`, still selects `metal`
0.33 and `paste` 1.0.15. The pinned revision can move, but moving it does not
resolve this advisory.

### RUSTSEC-2026-0192 (`ttf-parser` 0.25.1)

RUSTSEC classifies `ttf-parser` as unmaintained, not vulnerable, and lists no
patched versions. It is selected directly by GPUI and through
`usvg -> fontdb`; `rustybuzz` also uses it. Font parsing and text rendering are
part of libneo's actual GPUI use, so this dependency is runtime-reachable.

RUSTSEC recommends `skrifa`, which is a different API rather than an available
upgrade. The current Zed main revision still pins `ttf-parser` 0.25, so updating
the GPUI revision would not remove the advisory.

### RUSTSEC-2026-0206 (`rustybuzz` 0.20.1)

RUSTSEC classifies `rustybuzz` as unmaintained, not vulnerable, and lists no
patched versions. It enters through `gpui -> usvg -> rustybuzz`. libneo's own
code does not invoke GPUI's SVG renderer, so that runtime path is not reachable
from libneo's implementation. It remains available in the linked GPUI graph to
applications that render SVG assets.

RUSTSEC recommends `harfrust`, which requires the `usvg` dependency to migrate.
The current Zed main revision still selects `usvg` 0.46.0 and `rustybuzz`
0.20.1, so updating the GPUI revision would not remove the advisory.

cargo-deny 0.20.2 supports an advisory ID and a reason for each exception, but
has no enforceable per-ignore expiry field. `deny.toml` therefore records the
review deadline in each supported reason field. It also denies ignored
advisories that disappear from the graph, preventing obsolete exceptions from
remaining silently.

## License clarifications

`gpui_shared_string` 0.1.0 and `gpui_util` 0.1.0 omit `license` and
`license-file` metadata because their manifests inherit only publication and
edition settings from a workspace that defines no workspace license. The source
repository contains both Apache-2.0 and GPL-3.0-or-later license files and says
that its code is primarily GPL-3.0-or-later, with Apache-2.0 components where
marked. GPUI is marked Apache-2.0, but these two helper manifests are not.

The former cargo-deny clarifications asserted Apache-2.0 with empty
`license-files` arrays. They therefore had no file evidence and were not
supportable. Both clarifications were removed. Repository-owned compatibility
crates now provide the interfaces required by the pinned GPUI graph under
`MIT OR Apache-2.0`, with explicit package metadata. No upstream source was
relicensed.

## Stricter policy

The review enabled the following cargo-deny 0.20.2 settings:

- deny yanked crates;
- assess unmaintained and unsound advisories across the full graph;
- deny unused advisory ignores;
- deny wildcard version requirements;
- continue denying unknown registries and unknown Git sources.

Multiple-version diagnostics remain allowed because warning on every duplicate
would add 20 standing warnings to the configured macOS graph without identifying
an independently actionable update.

## Duplicate versions

The configured macOS graph currently has 20 crate names with two selected
versions each: `bitflags`, `getrandom`, `hashbrown`, `heck`, `itertools`,
`objc2`, `objc2-app-kit`, `objc2-foundation`, `object`, `png`, `pollster`,
`serde_spanned`, `shlex`, `spin`, `syn`, `thiserror`, `thiserror-impl`, `toml`,
`toml_datetime`, and `winnow`.

The `objc2` family is the most relevant to this crate: libneo and current GPUI
use the 0.6/0.3 generation, while GPUI's AccessKit dependency still uses the
0.5/0.2 generation. The remaining duplicates are split among GPUI runtime and
build dependencies. Cargo cannot unify any of these pairs within their current
semver requirements, and updating the GPUI pin to current main does not remove
the advisory dependencies. No duplicate can be resolved locally without
replacing or patching a transitive dependency, so none was changed in this
review.
