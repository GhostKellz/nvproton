# FFI Layer

The `src/ffi/` module provides Rust bindings to the native `nv*` libraries.
These are loaded dynamically at runtime, so nvproton runs even when some
libraries are absent — the corresponding features are simply skipped.

## Libraries

| Library | Purpose | Used by |
|---------|---------|---------|
| `nvshader` | Shader cache pre-warming with progress reporting | `prepare`, `run` |
| `nvlatency` | NVIDIA Reflex low-latency mode injection | `run --reflex` |
| `nvsync` | VRR / G-Sync management | `run --vrr` |

## Dynamic Loading

Libraries are opened with [`libloading`](https://docs.rs/libloading) rather than
link-time binding. Discovery order:

1. The directory in `NVPROTON_LIB_PATH`, if set.
2. Standard data/library locations under the user's data directories.
3. System default loader paths.

If a library or symbol is missing, the feature is reported as unavailable and
the launch continues without it.

## Safety

All `unsafe` related to the native ABI is confined to this module:

- C strings are constructed with `CString` and validated before crossing the
  boundary.
- Return codes are mapped into a typed `FfiError` and surfaced as normal Rust
  `Result`s.
- Handles obtained from the libraries are released on the matching teardown
  path.

When extending the FFI layer, keep `unsafe` blocks small and document the
invariants each one relies on (see [CONTRIBUTING.md](../../CONTRIBUTING.md)).

## Related

- [Architecture](architecture.md) — where the FFI layer sits in the run flow
- [Environment Variables](../reference/environment-variables.md) —
  `NVPROTON_LIB_PATH` and Reflex variables
