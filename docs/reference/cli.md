# CLI Reference

```
nvproton <COMMAND> [OPTIONS]
```

Global flags: `--version`, `--help`. Output-producing subcommands accept
`--format <text|json|yaml>` where noted.

## run

Run a game with NVIDIA optimizations.

```bash
nvproton run [GAME_ID] [OPTIONS] [-- GAME_ARGS...]
```

| Option | Description |
|--------|-------------|
| `GAME_ID` | Steam AppID or game identifier (positional) |
| `--name <NAME>` | Run game by name (fuzzy match) |
| `-p, --profile <NAME>` | Profile to apply |
| `--reflex` | Enable Reflex low-latency mode |
| `--fps <N>` | Target frame rate (0 = unlimited, default 0) |
| `--vrr` | Enable VRR (G-Sync/FreeSync) |
| `--no-prewarm` | Skip shader pre-warming |
| `--dry-run` | Show what would be done without launching |
| `--descriptor-heap <auto\|on\|off>` | `VK_EXT_descriptor_heap` mode for DX12 (default `auto`) |
| `-- <GAME_ARGS>` | Extra arguments passed to the game |

## prepare

Pre-warm shaders and set up a game profile before launch.

```bash
nvproton prepare [GAME_ID] [--name <NAME>] [-p <PROFILE>] [--force] [--progress]
```

## games

Manage detected games.

| Subcommand | Description |
|------------|-------------|
| `games list [--source <steam\|heroic\|lutris>] [--format]` | List detected games |
| `games show <GAME_ID>` | Show details for a game |
| `games scan [--all] [--fingerprint]` | Scan for new games |
| `games set-profile <GAME_ID> <PROFILE>` | Assign a profile to a game |
| `games info <GAME_ID> [--command]` | Show launch info / full command |
| `games dx12 [--installed] [--format]` | List DX12 games that benefit from descriptor_heap |

## detect

Detect games from launchers. Each accepts `--format`, `--update-db`, `--fingerprint`.

```bash
nvproton detect steam
nvproton detect heroic
nvproton detect lutris
nvproton detect all
```

## profile

Manage game profiles. See [Profiles](profiles.md).

| Subcommand | Description |
|------------|-------------|
| `profile list` | List profiles |
| `profile show <NAME>` | Show a profile |
| `profile create <NAME> [--base <NAME>] [--set KEY=VALUE]...` | Create a profile |
| `profile set <NAME> [--set KEY=VALUE]...` | Update profile values |
| `profile import <PATH> [--name <NAME>]` | Import a profile file |
| `profile export <NAME> [--format] [--path <PATH>]` | Export a profile |

## steam

Steam integration. See [Steam Integration](../guides/steam-integration.md).

| Subcommand | Description |
|------------|-------------|
| `steam launch-options <GAME_ID> [...]` | Generate optimized launch options |
| `steam proton list` | List installed Proton versions |
| `steam proton recommended` | Show recommended Proton versions for NVIDIA |
| `steam proton set-default <VERSION>` | Show how to set a default Proton |
| `steam shortcut create <NAME> <EXE> [...]` | Create a non-Steam shortcut |
| `steam shortcut list` | List non-Steam shortcuts |
| `steam shortcut optimize <APPID> [--profile <NAME>]` | Optimize a shortcut |

## preset

Built-in presets. See [Presets](presets.md).

| Subcommand | Description |
|------------|-------------|
| `preset list` | List available presets |
| `preset show <NAME>` | Show preset details |
| `preset install [--force]` | Install all presets as profiles |
| `preset recommend` | Recommend a preset for this system |

## mangohud

MangoHud config generation. See [MangoHud](../guides/mangohud.md).

| Subcommand | Description |
|------------|-------------|
| `mangohud status` | Check MangoHud installation |
| `mangohud generate [PRESET] [--output <PATH>] [--game <NAME>]` | Generate config |
| `mangohud env [PRESET]` | Show MangoHud environment variables |

## gamemode

Feral GameMode integration. See [GameMode](../guides/gamemode.md).

| Subcommand | Description |
|------------|-------------|
| `gamemode status` | Check GameMode install and daemon |
| `gamemode generate [TYPE] [--output <PATH>]` | Generate GameMode config |
| `gamemode prefix` | Show GameMode launch prefix |

## config

```bash
nvproton config show     # print configuration
nvproton config paths    # print resolved paths
nvproton config reset    # restore defaults
```

## status

```bash
nvproton status [--format <text|json|yaml>] [-v|--verbose] [--check]
```

`--check` exits 0 when the system is ready for `descriptor_heap`, 1 otherwise —
useful for scripting.
