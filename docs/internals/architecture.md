# Architecture

nvproton is a Rust CLI (and library) that sits between game launchers and the
NVIDIA Vulkan stack, applying optimizations and orchestrating the native `nv*`
tools.

```
┌─────────────────────────────────────────────────────────────┐
│                    Steam / Game Launcher                     │
├─────────────────────────────────────────────────────────────┤
│                        nvproton                              │
│   detect ──► profiles ──► runner ──► integrations ──► ffi    │
├───────────┬───────────┬───────────┬─────────────────────────┤
│ nvlatency │ nvshader  │  nvsync   │       nvvk               │
│  (reflex) │  (cache)  │  (vrr)    │    (vulkan ext)          │
├───────────┴───────────┴───────────┴─────────────────────────┤
│              Proton / Wine (DXVK + vkd3d-proton)             │
├─────────────────────────────────────────────────────────────┤
│                    NVIDIA Vulkan Driver                      │
└─────────────────────────────────────────────────────────────┘
```

## Module Layout

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Entry point; routes parsed CLI commands to handlers |
| `cli.rs` | `clap` command, subcommand, and argument definitions |
| `detection/` | Game/driver/tool discovery (see below) |
| `profile/` | Profile model, YAML manager, SQLite bindings, command handlers |
| `runner.rs` | Builds the launch context and executes the game |
| `steam.rs` | Launch-option generation, Proton management, shortcuts |
| `gamemode.rs` | Feral GameMode config generation and status |
| `mangohud.rs` | MangoHud config generation and environment output |
| `presets.rs` | Built-in optimization presets, including DLSS variants |
| `dx12_games.rs` | API classification database (DX12/DX11/Vulkan/OpenGL) |
| `cache.rs` | Shader-cache management (DXVK, vkd3d, NVIDIA GL, Mesa) |
| `config.rs` | Configuration model and resolved paths |
| `status.rs` | System status and driver-readiness reporting |
| `ffi/` | Bindings to the native `nv*` libraries (see [FFI Layer](ffi.md)) |

## Detection Subsystem (`detection/`)

| File | Role |
|------|------|
| `mod.rs` | Orchestration and output formatting (text/json/yaml) |
| `steam.rs` | Parses Steam `*.acf` manifests and locates executables |
| `heroic.rs` | Reads Heroic (Epic/GOG) configuration |
| `lutris.rs` | Reads Lutris game configuration |
| `vulkan.rs` | Detects Vulkan extensions and parses driver version/branch |
| `proton_nv.rs` | Finds and validates Proton-NV installations |
| `database.rs` | Loads/saves the persistent game database |
| `fingerprint.rs` | SHA-256 fingerprinting of executables |

## Profile Subsystem (`profile/`)

| File | Role |
|------|------|
| `model.rs` | `ProfileDocument` and `ResolvedProfile` types |
| `manager.rs` | Loads/saves YAML profiles and resolves inheritance |
| `persistence.rs` | SQLite backend for game-to-profile bindings |
| `mod.rs` | `profile` command handlers |

## Typical Run Flow

1. **Resolve game** — look up the game by AppID or name from the database.
2. **Resolve profile** — load the assigned/overridden profile and apply
   inheritance.
3. **Probe driver** — check Vulkan extensions to decide `descriptor_heap`,
   Reflex, and VRR availability.
4. **Prepare** — pre-warm shaders via `nvshader` (unless `--no-prewarm`).
5. **Build environment** — set `NVPROTON_*`, vkd3d-proton, and Reflex variables.
6. **Launch** — execute the game through Proton/Wine, optionally wrapped by
   GameMode and MangoHud.

## Persistence and Paths

- Configuration and profiles: `~/.config/nvproton/`
- Game/profile bindings: local SQLite database (`rusqlite`, bundled)
- Shader caches: `~/.cache/` (per backend)

See [Configuration](../getting-started/configuration.md) for resolved paths.
