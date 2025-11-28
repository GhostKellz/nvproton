# nvproton

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![NVIDIA](https://img.shields.io/badge/NVIDIA-76B900?style=for-the-badge&logo=nvidia&logoColor=white)](https://www.nvidia.com/)
[![Wayland](https://img.shields.io/badge/Wayland-FFBC00?style=for-the-badge&logo=wayland&logoColor=black)](https://wayland.freedesktop.org/)
[![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://www.linux.org/)
[![Vulkan](https://img.shields.io/badge/Vulkan-AC162C?style=for-the-badge&logo=vulkan&logoColor=white)](https://www.vulkan.org/)
[![Steam](https://img.shields.io/badge/Steam-000000?style=for-the-badge&logo=steam&logoColor=white)](https://store.steampowered.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)

**NVIDIA-Optimized Proton Integration Layer for Linux Gaming**

A comprehensive integration layer that bridges all nv* tools with Steam Proton/Wine, providing automatic game optimization, Reflex injection, shader management, and per-game configurations.

## Overview

nvproton is the "glue" that connects all NVIDIA Linux gaming tools with Proton:

- **Automatic Optimization** - Detects games and applies optimal settings
- **Reflex Integration** - Injects nvlatency into supported games
- **Shader Pre-warming** - Triggers nvshader before game launch
- **VRR Configuration** - Sets up nvsync per-game
- **Profile Management** - Unified game profiles across all nv* tools

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Steam / Game Launcher                     │
├─────────────────────────────────────────────────────────────┤
│                        nvproton                              │
│  ┌───────────┬───────────┬───────────┬─────────────────┐   │
│  │  detect   │  inject   │  config   │    profiles     │   │
│  │  (games)  │  (hooks)  │  (env)    │    (unified)    │   │
│  └───────────┴───────────┴───────────┴─────────────────┘   │
├───────────┬───────────┬───────────┬─────────────────────────┤
│ nvlatency │ nvshader  │  nvsync   │       nvvk              │
│  (reflex) │  (cache)  │  (vrr)    │    (vulkan ext)         │
├───────────┴───────────┴───────────┴─────────────────────────┤
│              Proton / Wine (DXVK + vkd3d-proton)            │
├─────────────────────────────────────────────────────────────┤
│                    NVIDIA Vulkan Driver                      │
└─────────────────────────────────────────────────────────────┘
```

## Features

### Automatic Game Detection

nvproton maintains a database of games with optimal configurations:

```bash
# Games are auto-detected from:
# - Steam library
# - Heroic Games Launcher
# - Lutris
# - Manual additions

nvproton games list
nvproton games info "Cyberpunk 2077"
```

### Per-Game Optimization

| Game | Reflex | Shader Prewarm | VRR | Special Config |
|------|--------|----------------|-----|----------------|
| Cyberpunk 2077 | Boost | Yes | 60Hz cap | RT optimizations |
| Elden Ring | On | Yes | Auto | Anti-stutter patches |
| CS2 | Ultra | N/A (native) | 240Hz+ | Competitive preset |
| Apex Legends | Boost | Yes | Auto | EAC compatibility |

## Usage

### CLI Tool

```bash
# Run game with automatic optimization
nvproton run "Cyberpunk 2077"

# Run with specific settings
nvproton run "Elden Ring" --reflex boost --fps-limit 60

# Run arbitrary executable
nvproton run --exe /path/to/game.exe

# Pre-launch preparation (shader prewarm, config setup)
nvproton prepare "Cyberpunk 2077"

# Show what would be applied
nvproton run --dry-run "Elden Ring"

# Create/edit game profile
nvproton profile edit "Elden Ring"

# Export profile for sharing
nvproton profile export "Elden Ring" --output elden-ring.json
```

### Steam Integration

```bash
# Add to Steam launch options:
nvproton run %command%

# Or with specific profile:
nvproton run --profile competitive %command%

# Environment variables for fine-tuning:
NVPROTON_REFLEX=boost NVPROTON_FPS_LIMIT=144 %command%
```

### Library API (Rust)

```rust
use nvproton::{GameProfile, ProtonRunner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runner = ProtonRunner::new()?;

    // Auto-detect game and apply settings
    let profile = runner.detect_game("/path/to/game.exe")?;

    // Customize if needed
    let profile = profile
        .with_reflex(ReflexMode::Boost)
        .with_fps_limit(144)
        .with_vrr(true);

    // Pre-warm shaders
    runner.prepare(&profile)?;

    // Launch with all optimizations
    runner.run(&profile)?;

    Ok(())
}
```

### C API (for Zig tools)

```c
#include <nvproton/nvproton.h>

nvproton_ctx_t* ctx = nvproton_init();

// Detect game
nvproton_profile_t* profile = nvproton_detect_game(ctx, "/path/to/game.exe");

// Configure
nvproton_set_reflex(profile, NVPROTON_REFLEX_BOOST);
nvproton_set_fps_limit(profile, 144);

// Run with all optimizations
nvproton_run(ctx, profile);

nvproton_cleanup(ctx);
```

## Game Database

nvproton includes a community-maintained database of game configurations:

```yaml
# ~/.config/nvproton/games/cyberpunk2077.yaml
name: "Cyberpunk 2077"
steam_appid: 1091500
executable: "Cyberpunk2077.exe"

optimization:
  reflex: boost
  fps_limit: null  # VRR handles this
  vrr: true
  shader_prewarm: true

dxvk:
  async: true
  hud: null

vkd3d:
  shader_cache: true

environment:
  DXVK_ASYNC: 1
  VKD3D_SHADER_CACHE_PATH: ~/.cache/vkd3d-proton/cyberpunk2077

notes: |
  Ray tracing works best with:
  - RT Medium preset
  - DLSS Quality or Balanced
  - Frame Generation disabled (adds latency)
```

## Environment Variables

| Variable | Description | Values |
|----------|-------------|--------|
| `NVPROTON_REFLEX` | Reflex mode | `off`, `on`, `boost`, `ultra` |
| `NVPROTON_FPS_LIMIT` | Frame rate limit | Integer (0 = unlimited) |
| `NVPROTON_VRR` | VRR mode | `on`, `off`, `auto` |
| `NVPROTON_PREWARM` | Shader pre-warming | `on`, `off` |
| `NVPROTON_PROFILE` | Named profile | Profile name |
| `NVPROTON_DEBUG` | Debug output | `0`, `1`, `2` |

## Building

```bash
# Build CLI and library (Rust)
cargo build --release

# Build with all features
cargo build --release --all-features

# Run tests
cargo test
```

## Installation

```bash
# From source
cargo install --path .

# Or copy binary
sudo cp target/release/nvproton /usr/local/bin/

# Install game database
mkdir -p ~/.config/nvproton/games
cp games/*.yaml ~/.config/nvproton/games/
```

## Related Projects

| Project | Purpose | Integration |
|---------|---------|-------------|
| **nvcontrol** | GUI control center | Profile editing, launcher |
| **nvlatency** | Reflex implementation | Injected into games |
| **nvshader** | Shader cache | Pre-warming on launch |
| **nvsync** | VRR manager | Per-game VRR config |
| **nvvk** | Vulkan extensions | Low-level hooks |

## Requirements

- NVIDIA GPU (GTX 900 series or newer)
- NVIDIA driver 535+
- Proton 8.0+ or Wine 8.0+
- Rust 1.70+ (for building)
- Steam, Heroic, or Lutris (for game detection)

## Why Rust?

nvproton uses Rust because:
- Integrates with nvcontrol (also Rust)
- Process management and IPC is ergonomic in Rust
- serde/toml/yaml for configuration parsing
- Can still call into Zig libraries via C ABI

## License

MIT License - See [LICENSE](LICENSE)

## Contributing

See [TODO.md](TODO.md) for the development roadmap.
