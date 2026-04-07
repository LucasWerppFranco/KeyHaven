# KeyHaven

KeyHaven is a simple password manager that runs locally on Linux. Originally, it was created to run in a Wayland desktop application environment, but this part of the project was not properly tested. This project was created to be a 100% open-source alternative for users and developers who want more autonomy for their password manager.

> ⚠️ **Attention:** 1 - I made over 2:45 A.M by me because I had insomnia and because I'm paranoid about privacy :D

> ⚠️ **Attention:** 2 - The /vault-desktop directory containing the Wayland app has not been properly tested; I do not recommend trying to run it on your machine yet

> ⚠️ **Attention:** 3 - For now, the only way to use this application is via CLI. I recommend testing the commands in the test directory first.

```
                    ==                     ==␍
                 <^\()/^>               <^\()/^>␍
                  \/  \/                 \/  \/␍
                   /__\      .  '  .      /__\ ␍
      ==            /\    .     |     .    /\            ==␍
   <^\()/^>       !_\/       '  |  '       \/_!       <^\()/^>␍
    \/  \/     !_/I_||  .  '   \'/   '  .  ||_I\_!     \/  \/␍
     /__\     /I_/| ||      -== + ==-      || |\_I\     /__\␍
     /_ \   !//|  | ||  '  .   /.\   .  '  || |  |\\!   /_ \␍
    (-   ) /I/ |  | ||       .  |  .       || |  | \I\ (=   )␍
     \__/!//|  |  | ||    '     |     '    || |  |  |\\!\__/␍
     /  \I/ |  |  | ||       '  .  '       || |  |  | \I/  \␍
    {_ __}  |  |  | ||                     || |  |  |  {____}␍
 _!__|= ||  |  |  | ||                     || |  |  |  ||  |__!_␍
 _I__|  ||__|__|__|_||                     ||_|__|__|__||- |__I_␍
 -|--|- ||--|--|--|-||     ████            ||-|--|--|--||= |--|-␍
  |  |  ||  |  |  | ||     █  ████████     || |  |  |  ||  |  |␍
  |  |= ||  |  |  | ||     ████    █ █     || |  |  |  ||= |  |␍
  |  |- ||  |  |  | ||                     || |  |  |  ||= |  |␍
  |  |- ||  |  |  | ||                     || |  |  |  ||- |  | ␍
 _|__|  ||__|__|__|_||:::::::::::::::::::::||_|__|__|__||  |__|_␍
 -|--|= ||--|--|--|-||:::::::::::::::::::::||-|--|--|--||- |--|-␍
  jgs|- ||  |  |  | ||:::::::::::::::::::::|| |  |  |  ||= |  | ␍
~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^~~~~~~~~~␍
```

## Project Structure

This is the project's folder structure so far; it's also available in [/doc](doc/README.md), where you can find the rest of the project documentation.

```
KeyHaven/
├── docs/                   # Documentation files
│   ├── cli/               # CLI documentation
│   ├── daemon/            # Daemon documentation
│   ├── database/          # Database schema and architecture
│   └── desktop/           # Desktop app documentation
├── test/                  # Docker-based testing environment
├── vault/                 # Legacy vault directory (pre-crate structure)
├── vault-cli/             # Command-line interface (Rust)
├── vault-core/            # Core library: crypto, storage, generator
├── vault-daemon/          # Background service for IPC
├── vault-desktop/         # Tauri-based desktop application
│   └── src-tauri/        # Rust backend for the desktop app
├── vault-extension/       # Browser extension (future)
├── Cargo.toml            # Workspace root configuration
└── README.md             # Project overview
```

### Directory Descriptions

| Directory | Purpose | Language |
|-----------|---------|----------|
| `docs/` | Technical documentation, guides, and architecture references | Markdown |
| `test/` | Docker Compose environment for integration testing | Docker/YAML |
| `vault-cli/` | Interactive CLI tool for vault management. Communicates with daemon via Unix socket. | Rust |
| `vault-core/` | **Core library** - encryption (AES-256-GCM), password hashing (Argon2id), SQLite storage, password generator, and search matching. Used by all other components. | Rust |
| `vault-daemon/` | Background service that holds the decrypted vault in memory. Handles IPC via Unix sockets with auto-lock timeout. | Rust |
| `vault-desktop/` | Cross-platform GUI application built with Tauri (React frontend + Rust backend) | Rust/TypeScript/React |
| `vault-extension/` | Browser extension for autofill (planned feature) | TBD |

## Thanks

I'd like to thank you for your interest in the project! Please take another look at the code, modify it, and contribute so your name can be listed here! Thank you so much for visiting this page :D !!!
