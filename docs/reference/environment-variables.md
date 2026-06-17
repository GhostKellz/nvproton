# Environment Variables

## nvproton Variables

Set these to override behavior or feed values through a launcher.

| Variable | Description | Values |
|----------|-------------|--------|
| `NVPROTON_REFLEX` | Reflex mode | `off`, `on`, `boost`, `ultra` |
| `NVPROTON_FPS_LIMIT` | Frame rate limit | integer (0 = unlimited) |
| `NVPROTON_VRR` | VRR mode | `on`, `off`, `auto` |
| `NVPROTON_PREWARM` | Shader pre-warming | `on`, `off` |
| `NVPROTON_PROFILE` | Named profile to apply | profile name |
| `NVPROTON_DEBUG` | Debug verbosity | `0`, `1`, `2` |
| `NVPROTON_LIB_PATH` | Custom search path for native `nv*` libraries | filesystem path |

## vkd3d-proton Variables (set automatically)

nvproton sets these for DX12 games based on detected driver capabilities. They
are listed for transparency and debugging.

| Variable | Description | When set |
|----------|-------------|----------|
| `VKD3D_CONFIG` | vkd3d-proton config flags | DX12 games on driver 595+ |
| `VKD3D_FEATURE_LEVEL` | DX12 feature level | DX12 games (default `12_2`) |
| `__GL_REFLEX` | NVIDIA Reflex enable | when `--reflex` is used |
| `__GL_REFLEX_MODE` | Reflex 2.0 mode | 595+ with `VK_NV_low_latency2` |

## Usage with Steam

Environment variables can be combined with the launch wrapper in Steam's launch
options:

```
NVPROTON_REFLEX=boost NVPROTON_FPS_LIMIT=144 nvproton run %command%
```

See [Steam Integration](../guides/steam-integration.md) for generating these
automatically with `nvproton steam launch-options`.
