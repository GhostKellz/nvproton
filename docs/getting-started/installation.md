# Installation

## Requirements

- NVIDIA GPU (Turing or newer recommended for full feature support)
- NVIDIA driver 535+; **595+ recommended** for DX12 descriptor-heap fixes
  (`VK_EXT_descriptor_heap`, `VK_NV_extended_sparse_address_space`)
- Proton 8.0+ or Wine 8.0+
- Steam, Heroic, or Lutris for automatic game detection
- Rust 1.96 (2024 edition) to build from source

Optional, for full functionality, the native `nv*` libraries:
`nvshader` (shader cache), `nvlatency` (Reflex), `nvsync` (VRR).

## Build from Source

```bash
git clone https://github.com/ghostkellz/nvproton.git
cd nvproton
cargo build --release
```

The binary is produced at `target/release/nvproton`.

## Install the Binary

```bash
# Install with cargo
cargo install --path .

# Or copy the release binary manually
sudo cp target/release/nvproton /usr/local/bin/
```

## Arch Linux (PKGBUILD)

A PKGBUILD is provided under [`packaging/`](https://github.com/ghostkellz/nvproton/tree/main/packaging):

```bash
cd packaging
makepkg -si
```

## Verify

```bash
nvproton --version
nvproton status
```

`nvproton status` reports Vulkan availability, vkd3d-proton, detected Proton-NV
installs, and DX12 readiness. See the [Quickstart](quickstart.md) for next
steps.
