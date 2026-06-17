# Configuration

nvproton stores its configuration, profiles, and game database under the
standard XDG directories.

## Paths

| Data | Location |
|------|----------|
| Config | `~/.config/nvproton/` |
| Profiles | `~/.config/nvproton/profiles/*.yaml` |
| Game database | `~/.config/nvproton/` (managed via `detect --update-db`) |
| Shader caches | `~/.cache/` (DXVK, vkd3d-proton, NVIDIA GL, Mesa) |

Show the exact resolved paths on your system:

```bash
nvproton config paths
```

## The `config` Command

```bash
nvproton config show    # print current configuration
nvproton config paths   # print resolved file/directory locations
nvproton config reset   # restore default configuration
```

## Library Discovery

nvproton loads the native `nv*` libraries (`nvshader`, `nvlatency`, `nvsync`)
at runtime. Override the search location with:

```bash
export NVPROTON_LIB_PATH=/opt/nvtools/lib
```

See [Environment Variables](../reference/environment-variables.md) for the full
list of `NVPROTON_*` settings.

## Profiles

Per-game and named profiles are YAML documents with optional inheritance. See
[Profiles](../reference/profiles.md) for the format and the `profile` command.
