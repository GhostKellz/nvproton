# Quickstart

This walks through detecting games, checking driver readiness, and launching a
title with NVIDIA optimizations.

## 1. Check System Status

```bash
nvproton status            # summary
nvproton status --verbose  # include driver branch and extension matrix
nvproton status --check    # exit 0 if DX12 (descriptor_heap) ready, 1 otherwise
```

## 2. Detect Your Games

Scan all configured launchers and store results in the local database:

```bash
nvproton detect all --update-db
```

Or target a single source:

```bash
nvproton detect steam --update-db
nvproton detect heroic
nvproton detect lutris
```

List what was found:

```bash
nvproton games list
nvproton games show 1245620          # by Steam AppID
nvproton games dx12                  # games that benefit from descriptor_heap
```

## 3. Prepare a Game

Pre-warm shaders and set up configuration before launch:

```bash
nvproton prepare 1245620
nvproton prepare --name "Elden Ring" --force
```

## 4. Run a Game

```bash
# By Steam AppID
nvproton run 1245620

# By name (fuzzy match)
nvproton run --name "Elden Ring" --reflex --vrr --fps 144

# Preview without launching
nvproton run --name "Elden Ring" --dry-run
```

## 5. Apply a Preset

Install the built-in presets as profiles, then run with one:

```bash
nvproton preset list
nvproton preset recommend          # suggest a preset for this system
nvproton preset install
nvproton run --name "CS2" --profile competitive
```

## Next Steps

- [Configuration](configuration.md) — manage config files and paths
- [Steam Integration](../guides/steam-integration.md) — generate launch options
- [Profiles](../reference/profiles.md) — customize per-game settings
