# V1.0 Clean Machine Validation

## Baseline

- Commit: `2589160 chore: ignore tauri build artifacts`
- Build command: `npm run tauri build`
- Build result: Success
- Artifact paths:
  - NSIS: `src-tauri/target/release/bundle/nsis/tauri-app_0.1.0_x64-setup.exe`
  - MSI: `src-tauri/target/release/bundle/msi/tauri-app_0.1.0_x64_en-US.msi`
  - EXE: `src-tauri/target/release/tauri-app.exe`
- Signing: Unsigned test build (no signing certificate configured)

## Local validation completed

| Command | Result |
|---------|--------|
| `npx vitest run src/App.test.tsx --reporter=verbose` | 14 tests passed |
| `npm run build` | 38 modules, built in ~800ms |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Passed |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 87 tests passed |
| `npm run tauri build` | Success, 3 artifacts generated |

## Clean machine validation matrix

| Scenario | Status | Notes |
|----------|--------|-------|
| Clean Windows 11 machine | **Pending manual validation** | Install app, launch, run detection, verify no crash |
| Windows with Node/Git installed but PATH abnormal | **Pending manual validation** | Verify `installed_but_path_missing` status and PATH repair instructions |
| Network-restricted Windows requiring system proxy or npm mirror | **Pending manual validation** | Verify winget proxy warning and npm registry setting |
| User with ccSwitch already installed | **Pending manual validation** | Verify manual path save and open ccSwitch |
| No ccSwitch download source configured | **Pending manual validation** | Verify empty source does not crash and manual path fallback works |
| Non-admin user | **Pending manual validation** | Verify admin hint and restart-as-admin flow |

## Manual validation steps

1. Install NSIS or MSI package on a clean Windows machine.
2. Launch app. Confirm window opens without crash.
3. Confirm no API Key is requested by this app at any point.
4. Run full environment detection (click "刷新检测").
5. Verify Git / Node / npm / Python statuses reflect actual machine state.
6. Verify Claude Code / Codex / OpenCode statuses reflect actual machine state.
7. Verify ccSwitch: if installed, save path and open; if not, confirm manual path fallback.
8. Verify node subscription page button only opens URL in browser.
9. Verify logs are displayed with 5000-line limit.
10. Export diagnostics zip. Confirm zip is created at expected path.
11. Confirm app does not modify system proxy (`netsh winhttp show proxy` before and after).
12. Confirm app does not modify npm global registry (`npm config get registry` before and after).
13. Confirm PATH repair only shows instructions and copy command; no automatic PATH modification.
14. Confirm app does not read or upload any API Key.
15. Close and re-launch app. Confirm config persists correctly.

## Known limitations

- **winget proxy**: winget does not reliably read `HTTP_PROXY` / `HTTPS_PROXY` environment variables. App-level manual proxy may not affect winget-based installs.
- **SystemProxy mode**: Does not mean this app configures the system proxy. It only inherits existing proxy environment variables.
- **ccSwitch download source**: Defaults to empty. Users must manually specify ccSwitch path or configure a DirectExe download source.
- **DirectExe only**: V1.0 ccSwitch download sources support DirectExe type only. DirectZip (archive extraction) is not implemented.
- **Unsigned builds**: All artifacts are unsigned unless a signing certificate is configured in `tauri.conf.json` or CI environment.
- **Windows-only**: This app targets Windows only. No macOS or Linux support.
- **Single install lock**: Only one tool can be installed at a time. Other install buttons are disabled during an active install.
- **UAC validation**: Some scenarios (admin restart, winget installs) require manual Windows UAC interaction that cannot be fully automated.

## Release readiness

- All local validation commands passed (frontend tests, frontend build, cargo check, cargo test, tauri build).
- Clean machine validation has **not** been performed yet. All scenarios in the matrix above are **Pending manual validation**.
- This build can be used as an **internal test release** for manual validation on clean Windows machines.
- Do not distribute as a production release until at least the first three clean machine scenarios are validated.
