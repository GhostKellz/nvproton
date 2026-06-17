# MangoHud

nvproton can check for MangoHud and generate overlay configurations from
built-in presets.

## Status

```bash
nvproton mangohud status
```

Reports whether MangoHud is installed and where its config is expected.

## Generate a Config

```bash
# Write a config from a preset (default: standard)
nvproton mangohud generate standard

# Choose a preset and output path
nvproton mangohud generate competitive --output ~/.config/MangoHud/MangoHud.conf

# Per-game config
nvproton mangohud generate full --game "Cyberpunk 2077"
```

Without `--output`, the config is written to
`~/.config/MangoHud/MangoHud.conf`.

## Presets

| Preset | Use |
|--------|-----|
| `minimal` | FPS only, smallest footprint |
| `compact` | FPS + frametime, small overlay |
| `standard` | Common metrics (default) |
| `full` | Detailed metrics for analysis |
| `steam-deck` | Tuned for the Steam Deck display |
| `competitive` | Latency-focused, minimal distraction |
| `debug` | Verbose metrics for troubleshooting |

## Environment Variables

To apply MangoHud through environment variables instead of a config file:

```bash
nvproton mangohud env competitive
```

This is also wired into `steam launch-options --mangohud`; see
[Steam Integration](steam-integration.md).
