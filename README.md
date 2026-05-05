# AI Coding 环境助手

A Windows desktop tool (Tauri + React + TypeScript + Rust) that helps users detect, install, and manage AI Coding CLI environments (Claude Code, Codex CLI, OpenCode, ccSwitch).

## Development

### Prerequisites

- Rust toolchain (via rustup)
- Node.js LTS
- Visual Studio Build Tools (for Tauri native builds on Windows)

### Commands

```powershell
npm install
npm run dev           # start Tauri dev window
npm run build         # frontend-only build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npx vitest run src/App.test.tsx --reporter=verbose
```

### Validation Commands

```powershell
npx vitest run src/App.test.tsx --reporter=verbose   # frontend tests
npm run build                                         # frontend TypeScript + Vite build
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml      # Rust type check
cargo test --manifest-path src-tauri/Cargo.toml       # Rust unit tests
```

### Windows Build Prerequisites

- **Node.js** LTS (tested with v24.x)
- **npm** (comes with Node.js)
- **Rust toolchain** (via rustup, default `stable` channel)
- **Visual Studio Build Tools** with C++ workload (provides MSVC linker and Windows SDK)
- **Windows SDK** (installed alongside VS Build Tools C++ workload)
- **WebView2 Runtime** (pre-installed on Windows 10 21H2+ and Windows 11; Tauri bundles a fallback for older systems)
- **Tauri CLI** (`@tauri-apps/cli` v2, installed as dev dependency via npm)
- **WiX Toolset v3** and **NSIS** are downloaded automatically by Tauri CLI on first build; no manual installation required

### Packaging

```powershell
npm run tauri build   # produces NSIS installer + MSI in src-tauri/target/release/bundle/
```

Build artifacts:

```text
src-tauri/target/release/tauri-app.exe                              # standalone executable
src-tauri/target/release/bundle/msi/tauri-app_0.1.0_x64_en-US.msi  # MSI installer
src-tauri/target/release/bundle/nsis/tauri-app_0.1.0_x64-setup.exe # NSIS installer
```

**Signing status**: These are unsigned test builds. No signing certificate is configured in this project. Distribute at your own risk.

## Architecture

- **Frontend** (`src/`): React + TypeScript; communicates with the Rust backend via Tauri `invoke` calls and `listen` event subscriptions.
- **Backend** (`src-tauri/src/`): Rust; handles tool detection, install orchestration, AppConfig persistence, and log management.
- **AppConfig** is stored at `%APPDATA%\ai-coding-installer\config.json`.

## Known Limitations (V1.0)

### System Proxy Mode (`system_proxy`)

- This client does **not** take over or set the system proxy.
- `system_proxy` mode relies on the host process inheriting existing proxy environment variables (e.g. `HTTP_PROXY`, `HTTPS_PROXY`) set by the OS or other software.
- It does **not** execute `netsh winhttp set proxy` or any equivalent command.
- **winget** does not read `HTTP_PROXY` / `HTTPS_PROXY` / `ALL_PROXY` environment variables, so winget-based installs (Git, Node.js, Python) may fail behind a proxy regardless of the proxy mode selected. The UI displays a warning when `manual_proxy` mode is active.
- For reliable proxy support with winget, enable a system-wide proxy in your VPN/proxy client before running winget installs.

### `set_tool_path` Command

- The PRD lists `set_tool_path(toolId, path)` as a planned Tauri command.
- In the current implementation, the ccSwitch executable path is saved via `update_config({ ccswitchPath })` rather than a dedicated `set_tool_path` command.
- The functionality (save path → re-detect → open) is fully implemented through the config patch mechanism.
- A dedicated `set_tool_path` command may be added in a future commit if strict PRD command-surface alignment is required.

### ccSwitch Download Sources

- V1.0 ccSwitch download sources support **DirectExe** type only.
- **DirectZip** is not supported; archive extraction is not implemented.
- The default download source list can be empty. Users can manually specify the ccSwitch executable path when no source is configured.
- Unknown third-party download accelerators are intentionally excluded.

### First Version Exclusions

The following features are explicitly out of scope for V1.0:

- No one-click install of all tools
- No node subscription client or subscription parsing
- No API Key storage, reading, or uploading
- No direct modification of Provider configuration
- No direct read/write of the ccSwitch database
- No automatic `outdated` version detection
- No automatic PATH modification (PATH repair instructions are provided as copy-paste commands only)
- No global `npm config set registry` modification (registry is passed as a single `--registry=` flag per install command)
- No `netsh winhttp set proxy` or system proxy takeover
