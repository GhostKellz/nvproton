# Game Detection

nvproton discovers installed games from Steam, Heroic, and Lutris, and maintains
a local database with metadata and optional executable fingerprints.

## Sources

| Source | What is read |
|--------|--------------|
| Steam | Library folders and `*.acf` app manifests |
| Heroic | Epic/GOG entries from the Heroic configuration |
| Lutris | Games from `~/.config/lutris` |

## Detecting

```bash
# Detect from a single source
nvproton detect steam
nvproton detect heroic
nvproton detect lutris

# Detect from everything
nvproton detect all
```

Common flags:

| Flag | Effect |
|------|--------|
| `--update-db` | Write detected games into the local database |
| `--fingerprint` | Compute SHA-256 fingerprints of game executables |
| `--format <text\|json\|yaml>` | Output format |

```bash
nvproton detect all --update-db --fingerprint --format json
```

## Working with Detected Games

```bash
nvproton games list                  # all games in the database
nvproton games list --source steam   # filter by source
nvproton games show 1245620          # details for one game
nvproton games scan --all --fingerprint   # rescan and fingerprint
nvproton games dx12                  # DX12 titles that benefit from descriptor_heap
```

## Fingerprints

Fingerprinting computes a SHA-256 hash of a game's executable. This is used to
identify a game reliably even if its install path changes, and to match
community game definitions.

## Assigning Profiles

Once games are detected, attach optimization profiles:

```bash
nvproton games set-profile 1245620 competitive
```

See [Profiles](../reference/profiles.md) and [Quickstart](../getting-started/quickstart.md).
