# Profiles

Profiles are YAML documents that hold optimization settings. They support
inheritance, can be assigned to games, and can be imported/exported for sharing.

Profiles live in `~/.config/nvproton/profiles/<name>.yaml`. Game-to-profile
bindings are stored in a local SQLite database.

## The `profile` Command

```bash
nvproton profile list
nvproton profile show competitive

# Create a profile, optionally inheriting from a base
nvproton profile create my-rpg --base quality --set display.vrr=true

# Update values on an existing profile
nvproton profile set my-rpg --set optimization.reflex=on --set optimization.fps_limit=120

# Import / export
nvproton profile import ./shared-profile.yaml --name borrowed
nvproton profile export competitive --format yaml --path ./competitive.yaml
```

`--set KEY=VALUE` uses dotted paths to address nested keys (e.g.
`display.refresh_rate=90`).

## Assigning a Profile to a Game

```bash
nvproton games set-profile 1245620 competitive
nvproton run 1245620            # uses the assigned profile
nvproton run 1245620 --profile quality   # override for this launch
```

## Format

```yaml
name: "Cyberpunk 2077"
extends: quality            # optional: inherit from another profile

optimization:
  reflex: boost             # off | on | boost | ultra
  fps_limit: null           # integer, or null for unlimited
  vrr: true
  shader_prewarm: true

dxvk:
  async: true

vkd3d:
  shader_cache: true

environment:
  DXVK_ASYNC: 1
  VKD3D_SHADER_CACHE_PATH: ~/.cache/vkd3d-proton/cyberpunk2077
```

## Inheritance

`extends` resolves a parent profile and merges its settings; the child's values
take precedence. Inheritance chains are followed without circular loops. Use it
to build a base profile (e.g. `quality`) and layer per-game tweaks on top.

See [Environment Variables](environment-variables.md) for variables that
profiles can set and that nvproton applies automatically.
