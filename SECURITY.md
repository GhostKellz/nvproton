# Security Policy

## Supported Versions

nvproton is an experimental project under active development. Security fixes
are applied to the `main` branch and the latest tagged release only.

| Version | Supported |
|---------|-----------|
| `main`  | Yes       |
| latest release | Yes |
| older releases | No |

## Reporting a Vulnerability

Please report security issues privately. **Do not open a public issue for
security vulnerabilities.**

1. Use GitHub's [private vulnerability reporting](https://github.com/ghostkellz/nvproton/security/advisories/new)
   to open a draft advisory.
2. Include:
   - A description of the issue and its impact
   - Steps to reproduce or a proof of concept
   - Affected version(s) or commit hash
   - Any suggested remediation

You will receive an acknowledgement of the report. Once triaged, a fix and a
coordinated disclosure timeline will be agreed upon before any public details
are shared.

## Scope

nvproton launches games through Proton/Wine and loads native `nv*` libraries
via FFI. Security-relevant areas include:

- **FFI / dynamic library loading** — `NVPROTON_LIB_PATH` and the discovery of
  `nvshader`, `nvlatency`, and `nvsync` libraries (`src/ffi/`).
- **Process launching** — environment construction and command assembly for
  game execution (`src/runner.rs`, `src/steam.rs`).
- **Filesystem access** — profile, config, database, and shader-cache paths
  under `~/.config/nvproton` and `~/.cache`.
- **Untrusted input** — imported profiles (YAML/JSON) and detected game
  metadata parsed from Steam/Heroic/Lutris.

## Dependency Security

- Dependencies are monitored by [Dependabot](https://github.com/ghostkellz/nvproton/security/dependabot)
  for known advisories, with automated security update PRs enabled.
- Run `cargo audit` locally to scan the lockfile against the RustSec advisory
  database before submitting changes.
