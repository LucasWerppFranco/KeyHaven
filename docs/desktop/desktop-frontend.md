# Desktop Frontend Documentation

The KeyHaven Desktop frontend is built with React, TypeScript, and Vite. It communicates with the Rust backend via Tauri's IPC bridge.

## Project Structure

```
vault-desktop/src/
├── api/               # Typed IPC wrappers
│   ├── vault.ts       # Vault lifecycle API
│   ├── entries.ts     # Password entry CRUD
│   ├── generator.ts   # Password generation
│   └── index.ts       # Re-exports
│
├── components/        # Reusable UI components
│   ├── EntryList.tsx      # List of password entries
│   ├── EntryForm.tsx      # Add/edit entry form
│   └── PasswordGenerator.tsx # Password generation tool
│
├── pages/            # Route-level components
│   ├── SetupPage.tsx   # First-time setup
│   ├── UnlockPage.tsx  # Unlock vault screen
│   └── VaultPage.tsx   # Main vault interface
│
├── App.tsx           # Root component with routing
├── main.tsx          # React entry point
└── styles.css        # Global styles
```

## API Layer

The `api/` directory contains typed wrappers around Tauri's `invoke()` function.

### Pattern

Each API module follows this pattern:

```typescript
import { invoke } from "@tauri-apps/api/core";

// Define TypeScript interfaces matching Rust types
export interface VaultEntry {
  id: number;
  title: string;
  username: string | null;
  password: string;
  // ...
}

// Export async functions that invoke commands
export async function listEntries(search?: string): Promise<VaultEntry[]> {
  return invoke("list_entries", { search });
}
```

### Error Handling

Commands that fail in Rust return rejected promises:

```typescript
try {
  await unlockVault(password);
} catch (e) {
  // e is the error string from Rust
  setError(e as string);
}
```

### Available APIs

#### `vault.ts`

| Function | Command | Description |
|----------|---------|-------------|
| `initVault(password)` | `init_vault` | Create new vault |
| `unlockVault(password)` | `unlock_vault` | Unlock existing vault |
| `lockVault()` | `lock_vault` | Lock vault (clear key) |
| `isUnlocked()` | `is_unlocked` | Check vault status |
| `vaultExists()` | `vault_exists` | Check if vault file exists |

#### `entries.ts`

| Function | Command | Description |
|----------|---------|-------------|
| `listEntries(search?)` | `list_entries` | Get all entries (filtered) |
| `getEntry(query)` | `get_entry` | Get single entry by ID/title |
| `addEntry(entry)` | `add_entry` | Create new entry |
| `updateEntry(update)` | `update_entry` | Modify existing entry |
| `deleteEntry(id)` | `delete_entry` | Remove entry |
| `copyPassword(id)` | `copy_password` | Copy to clipboard |

#### `generator.ts`

| Function | Command | Description |
|----------|---------|-------------|
| `generatePassword(options)` | `generate_password_cmd` | Create random password |
| `generatePassphrase(options)` | `generate_passphrase_cmd` | Create Diceware passphrase |
| `checkPasswordStrength(password)` | `check_password_strength` | Analyze strength |

## Components

### EntryList

Displays a list of password entries with search/filter.

**Props:**
```typescript
interface EntryListProps {
  entries: VaultEntry[];
  loading: boolean;
  onRefresh: () => void;
}
```

**Features:**
- Shows entry icon (first letter of title)
- Displays title, username, URL
- Copy and edit action buttons
- Empty state when no entries

### EntryForm

Form for adding or editing password entries.

**Props:**
```typescript
interface EntryFormProps {
  onSave: () => void;  // Called after successful save
}
```

**Fields:**
- Title (required)
- Username (optional)
- Password (required)
- URL (optional)
- Notes (optional)

### PasswordGenerator

Interactive password generation tool.

**Features:**
- Length slider (8-64 characters)
- Toggle symbols on/off
- Generate button
- Strength indicator (weak/medium/strong)
- Copy to clipboard

## Pages

### SetupPage

Shown when no vault exists. Collects a master password to initialize the vault.

**Validation:**
- Minimum 12 characters
- Password confirmation must match
- Displays warning that password cannot be recovered

### UnlockPage

Simple password input to unlock an existing vault.

**Features:**
- Password input
- Error display on invalid password
- Auto-focus on input field

### VaultPage

Main application interface after unlocking.

**Layout:**
```
┌──────────────────────────────────────┐
│  Logo    [Entries] [Generator]  Lock │  ← Header with navigation
├──────────────────────────────────────┤
│  [Search...          ] [+ Add]      │  ← Search bar
├──────────────────────────────────────┤
│  ┌────────────────────────────────┐  │
│  │ 🔑  GitHub    user@email...   │  │  ← Entry cards
│  │     [Copy] [Edit]              │  │
│  └────────────────────────────────┘  │
│  ┌────────────────────────────────┐  │
│  │ 🔒  Netflix   user@email...   │  │
│  │     [Copy] [Edit]              │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

**Tabs:**
- **Entries** — List and manage passwords
- **Generator** — Create new passwords

**Routes:**
- `/vault` — Entry list
- `/vault/new` — Add entry form
- `/vault/edit/:id` — Edit entry form

## Routing

The app uses React Router with conditional redirects based on vault state:

```typescript
// App.tsx
<Routes>
  <Route path="/" element={
    !hasVault ? <Navigate to="/setup" />
    : !unlocked ? <Navigate to="/unlock" />
    : <Navigate to="/vault" />
  } />
  <Route path="/setup" element={<SetupPage />} />
  <Route path="/unlock" element={<UnlockPage />} />
  <Route path="/vault/*" element={<VaultPage />} />
</Routes>
```

## Styling

The app uses CSS custom properties for theming:

```css
:root {
  --bg-primary: #0d1117;     /* Main background */
  --bg-secondary: #161b22;   /* Card backgrounds */
  --bg-tertiary: #21262d;    /* Input backgrounds */
  --text-primary: #e6edf3;   /* Primary text */
  --text-secondary: #8b949e; /* Secondary text */
  --accent: #58a6ff;         /* Buttons, links */
  --accent-hover: #79b8ff;   /* Hover states */
  --border: #30363d;         /* Borders */
  --success: #238636;        /* Success states */
  --error: #f85149;          /* Error states */
}
```

**Design choices:**
- Dark theme by default (password managers are often used in dim environments)
- High contrast for readability
- Card-based layout for entries
- Clear visual hierarchy

## Type Safety

The frontend uses TypeScript with strict mode enabled. Types are manually kept in sync with Rust types.

### Example Type Definition

```typescript
// Matching vault_core::VaultEntry
interface VaultEntry {
  id: number;              // i64 in Rust
  title: string;           // String in Rust
  username: string | null; // Option<String> in Rust
  password: string;
  url: string | null;
  notes: string | null;
  tags: string | null;
  created_at: number;      // i64 (unix timestamp)
  updated_at: number;
}
```

### Future: Generated Types

Tauri can generate TypeScript types automatically using `tauri-specta`. This would eliminate manual type syncing.

## Build Configuration

### Vite Config

```typescript
// vite.config.ts
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 1420,  // Fixed port for Tauri
    strictPort: true,
  },
});
```

### TypeScript Config

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  }
}
```

## Development Workflow

### Running in Development

```bash
cd vault-desktop
npm install
npm run tauri dev
```

### Making Changes

1. **API changes** — Update both Rust command and TypeScript wrapper
2. **Component changes** — Edit in `components/`, hot reload applies automatically
3. **Page changes** — Edit in `pages/`, routing updates automatically
4. **Style changes** — Edit `styles.css`, hot reload applies

### Debugging

- Use browser DevTools (Ctrl+Shift+I or Cmd+Opt+I) for frontend
- Check terminal output for Rust backend logs
- Use `console.log()` in frontend, `println!()` or `log::info!()` in Rust

## Security Considerations

### Password Handling

- Passwords are passed to Rust only during API calls
- Password fields use `type="password"` to prevent shoulder-surfing
- Clipboard is cleared after 30 seconds (handled by Rust backend)

### XSS Prevention

- React's JSX escaping prevents most XSS
- User input is never rendered as HTML
- URLs are validated before being used in `href`

### Memory Safety

- Frontend never stores the encryption key
- All decryption happens in Rust backend
- Auto-lock clears key from memory
