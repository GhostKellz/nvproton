# GameMode

nvproton integrates with [Feral GameMode](https://github.com/FeralInteractive/gamemode)
to apply CPU/GPU performance tuning while a game runs.

## Status

```bash
nvproton gamemode status
```

Reports whether GameMode is installed and whether the daemon is running.

## Generate a Config

```bash
# Default config
nvproton gamemode generate

# Choose a config type and output path
nvproton gamemode generate high-performance --output ~/.config/gamemode/gamemode.ini
```

Without `--output`, the config is written to
`~/.config/gamemode/gamemode.ini`.

## Config Types

| Type | Use |
|------|-----|
| `default` | Balanced GameMode behavior |
| `high-performance` | Maximize performance (governor, GPU clocks) |
| `power-save` | Conservative tuning for battery/thermals |
| `competitive` | Latency-focused tuning for esports |

## Launch Prefix

To wrap a command with GameMode manually:

```bash
nvproton gamemode prefix
# prints the prefix, e.g.: gamemoderun
```

GameMode can also be enabled directly in generated Steam launch options with
`steam launch-options --gamemode`; see
[Steam Integration](steam-integration.md).
