# Dependency resolution design

`cargo-stale` treats Cargo metadata as the source of truth for dependency
discovery.  The application does not attempt to recreate workspace expansion,
dependency aliases, inherited workspace requirements, or target-specific
dependency tables from raw TOML.

The pipeline has three boundaries:

1. `cargo::resolver` invokes Cargo metadata and returns normalized dependency
   records containing the displayed alias, the registry package name, the
   version requirement, source package, and dependency kind.
2. `api` resolves registry package names.  It first uses the local sparse
   index, then makes bounded online requests only for names absent from that
   index.  A lookup is explicitly `Found`, `NotFound`, or `Unavailable`.
3. `version::comparison` is a pure policy module.  It parses Cargo-compatible
   semver requirements and returns a structured status rather than inferring
   state while formatting output.

Reports consume the structured result.  An unavailable lookup is never
reported as up-to-date, and the process returns a non-zero exit status when a
partial result is produced.
