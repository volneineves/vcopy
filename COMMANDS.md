# VCopy Commands

VCopy can be used as an interactive terminal UI, a background clipboard daemon, or a direct command-line tool. This makes the storage layer usable today from scripts and leaves a clean path for a future graphical interface.

## Installation

For public releases, publish a compressed binary named with this format:

```text
vcopy-linux-x86_64.tar.gz
vcopy-linux-aarch64.tar.gz
```

Users can install the latest release with:

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/vcopy/main/scripts/install.sh | sh
```

Until the repository owner is final, the installer can be pointed at any GitHub repository:

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/vcopy/main/scripts/install.sh | VCOPY_REPO=OWNER/vcopy sh
```

To install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/vcopy/main/scripts/install.sh | VCOPY_REPO=OWNER/vcopy VCOPY_VERSION=v0.1.0 sh
```

The installer places the binary in:

```text
~/.local/bin/vcopy
```

Make sure `~/.local/bin` is in the user's `PATH`.

## Versioning and Updates

VCopy should use semantic versioning:

```text
MAJOR.MINOR.PATCH
```

- Increment `PATCH` for bug fixes.
- Increment `MINOR` for backwards-compatible features.
- Increment `MAJOR` for breaking CLI, config, storage, or automation changes.

Every release should have a matching Git tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Users can check their installed version with:

```bash
vcopy --version
```

Users can update to the latest release with the same installer command:

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/vcopy/main/scripts/install.sh | VCOPY_REPO=OWNER/vcopy sh
```

A future update-check feature can compare `vcopy --version` with the latest GitHub release. A good CLI shape would be:

```bash
vcopy update --check
vcopy update
```

The daemon or TUI can later show a non-blocking message when a newer release exists.

## Interactive UI

```bash
vcopy
```

Opens the terminal history picker. Use the arrow keys or `j`/`k` to move, `Enter` to copy the selected item, `/` to search, `dd` to delete, and `q` to quit.

## Direct History Commands

```bash
vcopy --list
vcopy --list -n 10
```

Prints history without opening the UI. Each line includes the item id, kind, timestamp, and preview. Use the id with `--delete`.

```bash
vcopy --search query
vcopy --search query -n 10
```

Searches text previews and image markers without opening the UI. Images are shown as `[image WIDTHxHEIGHT]`.

```bash
vcopy --delete 42
```

Deletes a single history item by id. If the item is an image, its stored image file is removed too.

```bash
vcopy --clear
vcopy clear
```

Clears all history. Stored image files managed by VCopy are removed as well.

## Daemon Commands

```bash
vcopy daemon
```

Runs the clipboard monitor in the foreground. This is mainly used by service/autostart integrations.

```bash
vcopy start
vcopy stop
vcopy restart
vcopy status
```

Manages the background daemon. The daemon records clipboard text, clipboard images, and new PNG screenshots saved under `~/Pictures/Screenshots`.

## Install Commands

```bash
vcopy install
vcopy uninstall
```

Installs or removes the autostart integration. VCopy prefers a user systemd service when available and falls back to XDG autostart when systemd is unavailable or fails.

## Configuration

```bash
vcopy config
```

Runs the interactive configuration wizard for terminal popup settings, the keyboard shortcut, history limit, and whether history should be cleared when the daemon starts.

The configuration file is stored at:

```text
~/.config/vcopy/config.toml
```

## Language

```bash
vcopy lang
vcopy lang en
vcopy lang pt
vcopy lang es
```

Shows or changes the UI language.

## Clipboard Content Types

VCopy currently supports:

- Text clipboard entries
- Image clipboard entries
- PNG screenshots saved in `~/Pictures/Screenshots`

Images are intentionally shown as metadata, not previews, so the CLI remains portable across terminals.
