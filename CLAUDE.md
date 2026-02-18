# Pass GUI

Desktop GUI for the `pass` Unix password manager, built with Tauri 2 (Rust backend) + Vue 3 (frontend).

## Commands
```bash
npm install           # Install frontend deps
npm run tauri dev     # Dev mode with hot reload
npm run tauri build   # Build production app
```

## Architecture
- `src-tauri/src/pass.rs` — Rust Tauri commands wrapping `pass` CLI calls
- `src-tauri/src/lib.rs` — Tauri app setup and command registration
- `src/stores/password.ts` — Pinia store managing tree state, selection, and search
- `src/components/` — TreeView, DetailPanel, Toolbar, SearchBar
- `src/dialogs/` — Modal dialogs: Add, Edit, Generate, Move, Grep

## Key Design Decisions
- Backend reads `~/.password-store/` directly for tree listing (fast, no GPG decrypt needed)
- All mutating operations go through `pass` CLI to preserve GPG encryption and git integration
- Batch delete sorts paths longest-first to remove children before parents
- Search/filter is client-side on the tree; Grep/Find use `pass grep`/`pass find` via CLI
- Clipboard uses `@tauri-apps/plugin-clipboard-manager`

## Dependencies
- Tauri 2, Vue 3, Pinia, TypeScript, Vite
- `pass` v1.7.4 must be installed at `/usr/local/bin/pass`
- GPG must be configured with the appropriate key for the password store
