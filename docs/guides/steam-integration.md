# Steam Integration

nvproton can generate optimized Steam launch options, inspect Proton versions,
and manage non-Steam shortcuts.

## Launch Options

Generate launch options for a game and apply them in Steam → Properties →
Launch Options:

```bash
nvproton steam launch-options 1245620 --reflex --vrr --fps 144
nvproton steam launch-options 1245620 --mangohud --gamemode --copy-format
```

| Flag | Effect |
|------|--------|
| `--use-nvproton` | Wrap the game with nvproton (default true) |
| `--reflex` | Enable Reflex low-latency mode |
| `--vrr` | Enable VRR (G-Sync/FreeSync) |
| `--fps <N>` | Target frame rate (0 = unlimited) |
| `--shader-cache` | Use a dedicated shader cache path |
| `--mangohud` | Enable MangoHud overlay |
| `--gamemode` | Enable Feral GameMode |
| `--env KEY=VALUE` | Add extra environment variables (repeatable) |
| `--copy-format` | Output in a copy-paste-friendly form for Steam |

A typical generated option uses the `%command%` wrapper pattern:

```
nvproton run %command%
```

## Proton Versions

```bash
nvproton steam proton list          # installed Proton versions
nvproton steam proton recommended   # NVIDIA-recommended versions
nvproton steam proton set-default GE-Proton9-20
```

`set-default` prints the steps to apply the version (nvproton does not modify
Steam state directly).

## Non-Steam Shortcuts

```bash
# Add a non-Steam game
nvproton steam shortcut create "My Game" /games/mygame/game.exe \
  --start-dir /games/mygame --launch-options "nvproton run %command%"

# List shortcuts
nvproton steam shortcut list

# Generate optimized settings for an existing shortcut
nvproton steam shortcut optimize 1234567890 --profile competitive
```

See [Environment Variables](../reference/environment-variables.md) for the
variables these options apply.
