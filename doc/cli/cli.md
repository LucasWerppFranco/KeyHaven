# KeyHaven CLI

The **vault-cli** (binary name: `keyhaven`) is the command-line interface for the KeyHaven password manager. It serves as a proof of concept for the `vault-core` library before building visual interfaces.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Commands](#commands)
- [Output for Pipes](#output-for-pipes)
- [Rofi/Wofi Integration](#rofiwofi-integration)
- [Security Features](#security-features)

---

## Overview

The CLI provides complete vault management through an intuitive command structure. It uses:

- **clap** with derive macros for command structure and automatic `--help` generation
- **rpassword** for secure password input (no echo to stdout)
- **comfy-table** for formatted tables
- **indicatif** for progress spinners
- **colored** for colored terminal output

---

## Installation

The CLI is built as part of the workspace:

```bash
cargo build --package vault-cli
```

The binary is available at `./target/debug/keyhaven`.

---

## Commands

### `init`

Initializes a new vault with a master password.

```bash
keyhaven init
```

- Creates the database at `~/.config/keyhaven/vault.db` (or specified `--db-path`)
- Prompts for master password (minimum 12 characters)
- Confirms password to prevent typos

---

### `unlock`

Unlocks the vault and stores the derived key in memory.

```bash
keyhaven unlock [--timeout 15m]
```

**Options:**
- `--timeout`: Session timeout (format: `30s`, `15m`, `1h`). Default: `15m`

**Note:** In this proof of concept, the key is stored in an environment variable (`KEYHAVEN_SESSION_KEY`). In production, the daemon would hold the key.

---

### `lock`

Immediately locks the vault, clearing the session key from memory.

```bash
keyhaven lock
```

---

### `list`

Lists all password entries.

```bash
keyhaven list [--search <query>] [--json]
```

**Options:**
- `-s, --search`: Filter entries by title/username
- `--json`: Output as JSON for scripting

**Interactive output:**
```
╭────┬─────────────┬────────────┬─────────────────┬────────────────╮
│ ID │ Title       │ Username   │ URL             │ Modified       │
├────┼─────────────┼────────────┼─────────────────┼────────────────┤
│ 1  │ GitHub      │ user@ex... │ github.com      │ 04/04/2025...  │
╰────┴─────────────┴────────────┴─────────────────┴────────────────╯
```

**Pipe output:**
```
GitHub	user@example.com	github.com
GitLab	user@example.com	gitlab.com
```

---

### `get`

Retrieves and displays a password entry.

```bash
keyhaven get <query> [--copy] [--show] [--field <field>]
```

**Options:**
- `--copy`: Copies password to clipboard (clears after 30s)
- `--show`: Displays password in plain text
- `--field`: Outputs only the specified field (for pipes)

**Examples:**
```bash
# Show entry details
keyhaven get github

# Copy password to clipboard
keyhaven get github --copy

# Output only the password (for scripts)
keyhaven get github --field password | wl-copy

# Output only the username
keyhaven get github --field username
```

**Available fields:** `password`, `username`, `title`, `url`, `notes`

---

### `add`

Interactively adds a new password entry.

```bash
keyhaven add [--url <url>] [--gen]
```

**Options:**
- `--url`: Pre-fill the URL field
- `--gen`: Auto-generate a password

**Interactive prompts:**
- Title (required)
- Username (optional)
- Password (or auto-generated with `--gen`)
- URL (optional)
- Notes (optional)

---

### `gen`

Generates a secure password or passphrase.

```bash
keyhaven gen [--length 20] [--symbols] [--copy]
keyhaven gen [--words 4] [--copy]
```

**Options:**
- `-l, --length`: Password length (default: 20)
- `--symbols`: Include special characters
- `--words`: Generate a Diceware-style passphrase (N words)
- `--copy`: Copy result to clipboard

**Examples:**
```bash
# Generate 20-character password
keyhaven gen

# Generate passphrase with 6 words
keyhaven gen --words 6

# Generate and copy
keyhaven gen --length 32 --symbols --copy
```

---

### `check`

Checks password strength and breach status via Have I Been Pwned.

```bash
keyhaven check <password>
```

**Features:**
- Local strength analysis (entropy calculation)
- HIBP breach check using k-anonymity

**Example output:**
```
Verifying password...

✓ Password strength: Strong

🔍 Checking HIBP...
⚠ Password found in 49141 breaches!
   This password is not secure. Change immediately!
```

**Note:** The HIBP API uses k-anonymity—only the first 5 characters of the SHA-1 hash are sent to the server.

---

### `rofi`

Opens a rofi/wofi selector for Hyprland/desktop integration.

```bash
keyhaven rofi [--type]
```

**Options:**
- `--type`: Types the password using `ydotool` instead of clipboard

**Hyprland configuration:**
```
bind = SUPER, P, exec, keyhaven rofi
bind = SUPER SHIFT, P, exec, keyhaven rofi --type
```

**Behavior:**
- Lists all entries in rofi/wofi
- Copies selected password to clipboard on selection
- Clipboard is cleared after 30 seconds
- Falls back to clipboard if `ydotool` is unavailable

---

## Output for Pipes

When the `--field` flag is used or output is redirected (not a TTY), the CLI produces clean output without colors, icons, or decorative text—only the raw value.

**Detection:**
```rust
use std::io::IsTerminal;

if std::io::stdout().is_terminal() {
    // Interactive: use colors and formatting
} else {
    // Pipe: clean output
}
```

**Example:**
```bash
# In script: copies only the password
keyhaven get github --field password | wl-copy

# Output: just the password, no newline extra
mysecretpassword123
```

---

## Rofi/Wofi Integration

The `rofi` command provides desktop integration for Linux environments:

1. **Listing:** Formats entries as `title: username` for the launcher
2. **Selection:** Captures the selected index from rofi/wofi
3. **Action:** Copies or types the password
4. **Cleanup:** Schedules clipboard clearing after 30 seconds

**Requirements:**
- `rofi` or `wofi` installed
- `wl-copy` (Wayland) or `xclip` (X11) for clipboard
- `ydotool` (optional) for typing mode

---

## Security Features

| Feature        | Implementation                                  |
|----------------|-------------------------------------------------|
| Password input | `rpassword` - no echo to terminal                 |
| Session key    | Stored with `Zeroizing` wrapper (cleared on drop) |
| Clipboard      | Auto-cleared after 30 seconds                   |
| HIBP check     | k-anonymity (only 5 chars of SHA-1 sent)        |
| Pipe output    | Clean, no metadata leakage                      |
| Color output   | Disabled when not a TTY                         |

---

## Global Options

All commands support:

- `-d, --db-path <PATH>`: Custom database path
- `-s, --socket-path <PATH>`: Custom daemon socket path

**Examples:**
```bash
keyhaven -d /tmp/test.db init
keyhaven -d /tmp/test.db -s /tmp/test.sock unlock
```
