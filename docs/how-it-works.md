# How KeyHaven Works (For Users)

**KeyHaven** is a password manager that works like a digital safe with a smart lock. Here's how you use it day-to-day:

---

## 1. Setting Up (One Time)

When you first run KeyHaven, you create a **vault** — an encrypted database where all your passwords are stored:

```bash
# Initialize your vault with a master password
keyhaven init
# Enter your master password: **********
```

This creates:
- An encrypted database file (`~/.local/share/vault/vault.db`)
- A configuration file (`~/.config/vault/config.toml`)
- The **daemon** starts running in the background

---

## 2. Daily Usage Workflow

### Unlocking the Vault

Before you can access any passwords, you must **unlock** the vault:

```bash
keyhaven unlock
# Enter master password: **********
# Vault unlocked (auto-locks after 15 min of inactivity)
```

When you unlock:
- Your master password is never stored
- Instead, the daemon derives an encryption key and holds it in memory
- The key is wiped from memory if the daemon crashes or is killed

### Using Your Passwords

Once unlocked, you can:

```bash
# Add a new password
keyhaven add github
# Username: myuser
# Password: ********
# URL: https://github.com

# Retrieve a password (copied to clipboard)
keyhaven get github
# Password copied to clipboard (clears after 30 seconds)

# List all entries
keyhaven list
# Search: "git"
# 1. github - myuser

# Generate a strong password
keyhaven generate --length 20
# Generated: aB3$k9!mP2@qR7&xL4
```

### Locking

The vault **automatically locks** after 15 minutes of inactivity. Or you can lock it manually:

```bash
keyhaven lock
```

When locked, the encryption key is wiped from memory. Your passwords are inaccessible until you unlock again.

---

## 3. The Security Model

**Why a daemon?** Instead of each command loading the vault from disk (slow and risky), the daemon keeps the vault ready in memory while maintaining security:

```
┌─────────────┐     locked      ┌─────────────┐
│   You       │ ──────────────→ │ Encrypted   │
│             │                 │ Database    │
│             │ ←────────────── │ (Safe)      │
└─────────────┘    unlock       └─────────────┘
      ↓                              ↑
      │                              │
      └──────→  Daemon  ←────────────┘
              (Holds key only
               when unlocked)
```

**Key security features:**
- **Master password** → Never stored anywhere. Used only to derive the encryption key.
- **Argon2id** → Industry-standard key derivation that resists brute-force attacks (takes ~100ms to verify)
- **AES-256-GCM** → Military-grade encryption for each password entry
- **Auto-lock** → Key wiped from RAM after inactivity (prevents memory dumps)
- **Unix socket** → Only your user account can talk to the daemon

---

## 4. Multiple Ways to Access

The daemon accepts connections from:
- **CLI** (`keyhaven` command)
- **GUI apps** (can be built to connect via the socket)
- **Browser extensions** (via native messaging)

They all talk to the same daemon, so unlocking in one unlocks for all.

---

## 5. What Happens When...

| Scenario | What Happens |
|----------|-------------|
| Computer sleeps | Vault stays unlocked (key in memory) |
| Daemon crashes | Vault locks immediately (key lost from RAM) |
| Wrong password | Takes ~100ms to fail (prevents guessing) |
| Copy password | Clears from clipboard after 30 seconds |
| Auto-lock triggers | Key zeroed in memory, you must unlock again |

---

## Summary

KeyHaven works like a physical safe with a time-lock:

1. **Initialize once** → Creates encrypted vault
2. **Unlock** → Safe opens, key held in memory only
3. **Use** → Add/retrieve passwords instantly
4. **Auto-lock** → Safe closes after inactivity, key destroyed

Your passwords are encrypted at rest (in the file) and only decrypted temporarily while the vault is unlocked. The master password never leaves your head.
