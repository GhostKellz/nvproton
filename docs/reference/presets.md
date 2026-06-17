# Presets

Presets are ready-made optimization profiles for common scenarios. Install them
as profiles, then apply with `run --profile <name>`.

```bash
nvproton preset list
nvproton preset show competitive
nvproton preset recommend         # suggest one for this system
nvproton preset install [--force] # write all presets as profiles
```

## Available Presets

| Name | Description | GPU |
|------|-------------|-----|
| `steam-deck` | Optimized for Steam Deck handheld gaming | — |
| `competitive` | Ultra-low latency for esports titles | — |
| `balanced` | Good mix of performance and visual quality | — |
| `quality` | Maximum visual quality, performance secondary | — |
| `battery` | Power-efficient settings for extended battery life | — |
| `dlss-quality` | DLSS Quality mode — best image quality | RTX 20+ |
| `dlss-performance` | DLSS Performance — 2x upscaling with frame gen | RTX 40+ |
| `dlss-framegen` | DLSS 3 Frame Generation enabled | RTX 40+ |
| `dlss-mfg-4x` | DLSS 4 Multi Frame Gen 4x | RTX 50 |
| `dlss-dynamic` | DLSS 4.5 Dynamic MFG — adapts to display | RTX 50 |
| `dlss-max-fps` | DLSS 4.5 Max FPS — 6x frame gen for 4K@240Hz | RTX 50 |

## Aliases

Several presets accept alternate names:

| Preset | Aliases |
|--------|---------|
| `steam-deck` | `steamdeck`, `deck` |
| `competitive` | `esports`, `low-latency` |
| `balanced` | `default` |
| `quality` | `high`, `ultra` |
| `battery` | `power-save`, `powersave` |
| `dlss-framegen` | `dlss-fg` |
| `dlss-mfg-4x` | `mfg-4x`, `mfg4x` |
| `dlss-dynamic` | `dynamic` |
| `dlss-max-fps` | `max-fps` |

## Using a Preset

```bash
# Install presets as profiles, then run with one
nvproton preset install
nvproton run --name "CS2" --profile competitive
```

Installed presets become regular profiles and can be customized or extended.
See [Profiles](profiles.md).
