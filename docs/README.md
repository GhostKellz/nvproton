# nvproton Documentation

nvproton is an NVIDIA-optimized integration layer that bridges the `nv*` Linux
gaming tools (`nvlatency`, `nvshader`, `nvsync`, `nvvk`) with Steam Proton/Wine.
It handles automatic game detection, Reflex injection, shader pre-warming, VRR
configuration, and per-game profiles.

> Experimental project under active development. Use at your own risk.

## Getting Started

- [Installation](getting-started/installation.md) — build from source, install the binary, set up the database
- [Quickstart](getting-started/quickstart.md) — detect games, check status, launch your first title
- [Configuration](getting-started/configuration.md) — config files, paths, and the `config` command

## Reference

- [CLI Reference](reference/cli.md) — every command, subcommand, and flag
- [Presets](reference/presets.md) — built-in optimization presets including DLSS 4.5
- [Profiles](reference/profiles.md) — profile format, inheritance, import/export
- [Environment Variables](reference/environment-variables.md) — `NVPROTON_*` and vkd3d-proton variables

## Guides

- [Driver Support](guides/driver-support.md) — driver tiers, DX12 extensions, and detection
- [Steam Integration](guides/steam-integration.md) — launch options, Proton management, shortcuts
- [Game Detection](guides/game-detection.md) — Steam, Heroic, and Lutris scanning
- [MangoHud](guides/mangohud.md) — overlay config generation
- [GameMode](guides/gamemode.md) — Feral GameMode integration

## Internals

- [Architecture](internals/architecture.md) — module layout and data flow
- [FFI Layer](internals/ffi.md) — bindings to the native `nv*` libraries

## Project Info

- [Changelog](../CHANGELOG.md)
- [Contributing](../CONTRIBUTING.md)
- [Security Policy](../SECURITY.md)
