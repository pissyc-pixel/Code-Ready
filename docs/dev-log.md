# V0.1 验收记录

## 验收时间

2026-05-02 16:09 (Asia/Shanghai)

## 验收命令

### 1. 单元测试

命令：

```powershell
npx vitest run src/App.test.tsx
```

结果：

通过。

摘要：

```text
Test Files 1 passed
Tests 1 passed
```

### 2. 前端构建

命令：

```powershell
npm run build
```

结果：

通过。

摘要：

```text
tsc && vite build 成功
dist/ 产物已生成
29 modules transformed
built in 751ms
```

### 3. Rust / Tauri 检查

命令：

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

结果：

通过。

摘要：

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 56.91s
```

## 结论

Windows Rust/Tauri 编译前置已经验证通过，不再卡在 `link.exe` / Build Tools / Rust toolchain 环境问题上。

V0.1 当前状态：

- Mock 首页已完成。
- 单元测试通过。
- 前端 build 通过。
- `cargo check` 通过。

下一步：

进入 V0.1 阶段复盘与 git commit。

## PUA 阶段复盘

- pua 是否可用：可用
- 关键提醒：
  - 不要跳过验收，必须用真实命令给出证据
  - 不要把 mock 页面描述成真实检测闭环
  - 不要为了赶进度跨到 V0.2 或削弱 PRD 约束
  - 不要把未来安装能力做成长时间阻塞 invoke
  - 不要把环境问题伪装成“代码已经完成”

### V0.1 自检清单

1. 是否跳过验收：否，已执行单元测试、前端构建、Rust/Tauri 检查
2. 是否用 mock 冒充真实检测：否，当前明确标注为 Mock 首页，仅完成 V0.1 骨架
3. 是否偏离 PRD：否，仍保持 Windows-only、无一键全装、无 V0.2/V0.3 业务实现
4. 是否把安装写成长时间阻塞 invoke：否，本阶段未实现安装逻辑
5. 是否保存、读取或上传 API Key：否
6. 是否接管系统代理：否
7. 是否直接读写 ccSwitch 数据库：否
8. 是否做了一键全装：否
9. 是否做了自动 outdated 判断：否
10. 是否为了赶进度删除核心约束：否

# V0.2 Phase 3 Validation

## Time

2026-05-02 16:44 (Asia/Shanghai)

## Scope

- Implement AI tool detectors for `claude`, `codex`, `opencode`, and `ccswitch`.
- Emit `detect:result` from Rust during serial `detect_all_tools()`.
- Switch the frontend from static mock-only rendering to `checking` initial state plus real event-driven updates.
- Keep V0.2 detection-only: no install commands, no cancel flow, no ccSwitch download, no PATH repair.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 1 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
36 modules transformed
dist/ artifacts generated
built in 822ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 1.56s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
7 tests passed
0 failed
```

## Key Decisions

- `detect_all_tools()` remains serial for stability and emits one `detect:result` event per finished tool.
- `ccswitch` uses path probing only in V0.2 and never launches the GUI during automatic detection.
- `claude` / `codex` / `opencode` treat auth-related non-zero exits as installed or path-missing, not broken.
- Removed the `D:\nodejs\node.exe` personal-path fallback from the Node detector so the common rule stays Windows-generic.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not claim completion without command output.
  - Do not let mock UI masquerade as real detection.
  - Do not drift into install behavior during V0.2.
  - Do not weaken PRD constraints just to simplify implementation.
- Self-check result:
  - Skipped validation: no
  - Used mock instead of real detection: no
  - Drifted outside V0.2 scope: no
  - Introduced blocking install invoke: no
  - Saved/read/uploaded API key: no
  - Took over system proxy: no
  - Read/wrote ccSwitch database directly: no
- Added one-click install: no

# V0.4 Commit 3 Validation

## Commit Goal

- Commit target: `feat: add ccswitch path and launch support`
- Scope:
  - add `ccswitchPath` to AppConfig
  - prefer configured path in `ccSwitch` detection
  - add `open_ccswitch`
  - add frontend path save / re-detect / open UI
- Not included:
  - ccSwitch download
  - download sources
  - database writes
  - Provider writes
  - subscription parsing

## Difficulty Handling Record

### Current phase

- V0.4 Commit 3 implementation and acceptance

### Current task

- Add manual ccSwitch path persistence and open support without turning detection into GUI launch or drifting into download logic

### Failed commands

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
npx vitest run src/App.test.tsx --reporter=verbose
npm run build
```

### Error summary

- `cargo check` initially failed because `commands::ccswitch` tried to import the private detector submodule directly.
- A later parallel frontend verification pass hit the known `vitest` worker timeout / stale `node.exe` issue and left a hanging `vite build`.

### superpower

- Available: no
- Record:

```text
superpower 不可用。
已改为 findskills + 仓库文档 + 保守实现方案。
```

### findskills

- Available: yes
- Queries:

```powershell
npx skills find "vite vitest windows worker timeout stale node process"
```

- Relevant results:
  - `vitest`
  - `electron-app-dev`
- Adopted guidance:
  - keep the Rust fix minimal with a `pub(crate)` wrapper instead of exposing the whole detector module
  - treat the frontend timeout as execution noise and re-run tests serially after clearing only repository-related `node.exe` processes

### Final fix

- Added a minimal `detector::resolve_ccswitch_path(...)` wrapper for `open_ccswitch`.
- Kept `ccSwitch` detection path-only and GUI-free.
- Cleared only `D:\aicoding`-related stale `vite / vitest` node processes.
- Re-ran frontend verification serially.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 7 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 775ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 2.20s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
43 tests passed
0 failed
```

## Key Decisions

- `ccswitchPath` is now stored in the existing AppConfig at `%APPDATA%\ai-coding-installer\config.json`.
- `ccSwitch` detection now prefers the configured path first, then falls back to common install paths.
- Detection still never launches the GUI.
- `open_ccswitch` only runs when the user explicitly clicks the button and only launches an executable path.
- No `ccSwitch` database access, no Provider writes, and no download-source logic were introduced in this commit.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not confuse manual path save with “ccSwitch installed”.
  - Do not launch GUI during detection.
  - Do not drift into download or Provider/database work.
  - Re-run full acceptance after both the Rust wrapper fix and the frontend worker-timeout cleanup.
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 3 scope: no
- Added download logic: no

# V0.4 Commit 4 Validation

## Commit Goal

- Commit target: `feat: add ccswitch download source support`
- Scope:
  - add `ccswitchDownloadSources` config structure
  - order download sources by ascending `priority`
  - skip disabled or blank-url sources
  - retry next source after a failure
  - validate downloaded file existence, minimum size, and optional sha256
  - fail safely with a manual-path fallback message when all sources fail
- Not included:
  - unknown third-party accelerators
  - hard-coded uncertain official release links
  - database writes
  - Provider writes
  - deep link / node client work

## Difficulty Handling Record

### Current phase

- V0.4 Commit 4 implementation and acceptance

### Current task

- Build a safe ccSwitch download-source framework without guessing unstable release URLs or drifting into database/provider logic

### Failed commands

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

### Error summary

- `cargo test` initially failed because:
  - `installer::ccswitch` tests were missing a `PathBuf` import
  - several installer tests needed the new `ccswitchDownloadSources` field in `AppConfig`
- `npm run build` timed out in the same pass due to the recurring stale `vite` process issue, not because of a TypeScript regression

### superpower

- Available: no
- Record:

```text
superpower 不可用。
已改为 findskills + 仓库文档 + 保守实现方案。
```

### findskills

- Available: yes
- Query:

```powershell
npx skills find "vite vitest windows worker timeout stale node process"
```

- Relevant results:
  - `vitest`
  - `electron-app-dev`
- Adopted guidance:
  - keep Rust fixes limited to test scaffolding and config compatibility
  - clear only `D:\aicoding`-related stale `node.exe` processes
  - re-run frontend verification serially

### Final fix

- Added the missing test import and completed `AppConfig` initializers with the new field.
- Escaped PowerShell `{0}/{1}` placeholders in the generated script so Rust `format!` would not corrupt the command text.
- Cleared only repository-related stale `vite` processes and re-ran frontend verification serially.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 8 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 756ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 1.81s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
46 tests passed
0 failed
```

## Key Decisions

- `ccswitchDownloadSources` is now part of AppConfig, but the default value is an empty array.
- The project does not guess an official default download URL when the stable link is still uncertain.
- No unknown third-party GitHub accelerator was added.
- The generated download command validates:
  - file existence
  - minimum file size
  - optional sha256
- When all configured sources fail, the installer returns a clear fallback message instructing the user to save a manual `ccswitchPath`.
- `direct_zip` is represented in the schema for future compatibility, but V0.4 still tells the user to extract manually and save a path instead of pretending full zip installation is done.

## TODO

- Confirm an official, stable, auditable ccSwitch release/download URL before shipping any non-empty default source list.
- Add real archive extraction only after the source format and trust boundary are confirmed.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not fill in unverified release URLs just to look complete.
  - Do not use unknown third-party accelerators.
  - Do not drift into database or Provider writes.
  - Keep “all sources failed -> manual path fallback” explicit and real.
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 4 scope: no
  - Added unknown accelerator: no
  - Hard-coded uncertain official URL: no
  - Touched database or Provider: no
- Blocked future V0.5 work: no

# V0.5 Commit 1 Validation

## Commit Goal

- Commit target: `feat: add reinstall and latest install actions`
- Scope:
  - add `reinstall_tool`
  - add `install_latest_tool`
  - both still return immediately
  - final result still flows through `install:status`
  - npm AI tools use `@latest` in latest mode
- Not included:
  - auto outdated judgment
  - PATH repair instructions
  - subscription entry
  - log export

## Key Decisions

- `install_tool`, `reinstall_tool`, and `install_latest_tool` now share the same background install entrypoint and lock semantics.
- `install_latest_tool` is only enabled for npm-based AI tools:
  - `claude`
  - `codex`
  - `opencode`
- `claude` latest mode uses npm `@latest` to satisfy the V0.5 requirement without changing the global npm registry.
- `git / node / python / ccswitch` do not pretend to support “latest” in V0.5; they return a clear unsupported message instead.
- No auto outdated detection was introduced.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 9 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 921ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 1.97s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
50 tests passed
0 failed
```

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - keep reinstall/latest on the existing event-driven background runner
  - do not turn latest into outdated detection
  - do not widen latest support by guessing version channels for non-npm tools
  - do not skip full acceptance after adding new commands and UI actions
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 1 scope: no
  - Added auto outdated judgment: no
  - Broke immediate-return semantics: no
  - Persisted npm registry: no
  - Mixed in V0.5 commit 2-4 work: no
  - Touched database or Provider: no
  - Launched GUI during detection: no
  - Added node client logic: no

# V0.4 Commit 2 Validation

## Commit Goal

- Commit target: `feat: implement claude codex opencode installers`
- Scope:
  - Claude Code installer
  - Codex CLI installer
  - OpenCode installer
  - Claude `manual_proxy` prefers npm fallback
  - Claude native install uses PowerShell `-ExecutionPolicy Bypass`
  - Single-use npm registry flag reuse
  - Post-install re-detect for `claude / codex / opencode`
- Not included:
  - ccSwitch path/open/download
  - Provider writes
  - API key storage
  - PATH auto repair
  - one-click install

## Difficulty Handling Record

### Current phase

- V0.4 Commit 2 implementation and acceptance

### Current task

- Wire real AI installer command specs into the existing background install runner without expanding into ccSwitch

### Failed commands

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
apply_patch
```

### Error summary

- `cargo test` initially failed because unit tests for Claude / Codex / OpenCode were coupled to the local machine having npm available.
- A follow-up `apply_patch` attempt failed due to a malformed multi-file patch hunk, not because of a product design issue.

### superpower

- Available: no
- Record:

```text
superpower 不可用。
已改为 findskills + 仓库文档 + 保守实现方案。
```

### findskills

- Available: yes
- Query:

```powershell
npx skills find "rust test helper visibility module private function unit test"
```

- Relevant results:
  - `rust-refactor-helper`
  - `quality-unit-testing`
  - `rust-testing-code-review`
- Adopted guidance:
  - Keep the fix minimal.
  - Separate environment-dependent prerequisite checks from pure command-construction tests.
  - Re-run the full acceptance suite after the small visibility/testability fix.

### Final fix

- Split “command construction” from “runtime prerequisite enforcement” for Claude / Codex / OpenCode unit tests.
- Kept real runtime prerequisite checks in public installer entrypoints.
- Added the missing internal helper imports in the test modules.
- Re-ran the full acceptance suite after the fix.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 5 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 754ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 6.52s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
38 tests passed
0 failed
```

## Key Decisions

- Claude installer behavior now matches the PRD:
  - `manual_proxy` -> prefer npm fallback
  - otherwise -> use official PowerShell native install with `-ExecutionPolicy Bypass`
- Codex and OpenCode install through global npm only, with single-use `--registry` when configured.
- No `npm config set registry`, no system proxy takeover, and no Provider or API key writes were introduced.
- Manual proxy values are applied only as per-process environment variables, not as system-wide proxy changes.
- Frontend now exposes install buttons for `claude / codex / opencode`, while `ccSwitch` remains out of scope for this commit.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not let this commit drift into ccSwitch.
  - Do not turn `install_tool` into a blocking invoke.
  - Do not fake AI installers with mock-only behavior.
  - Do not persist npm registry or system proxy changes.
  - Fix testability issues without deleting real prerequisite checks.
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 2 scope: no
  - Mixed in ccSwitch work: no
  - Introduced blocking install invoke: no
  - Persisted npm registry: no
  - Modified system proxy: no
  - Saved or read API keys: no
  - Added one-click install: no
  - Added automatic outdated judgment: no
  - Removed core PRD constraints: no

# V0.3 Commit 1 Validation

## Time

2026-05-02 17:10 (Asia/Shanghai)

## Scope

- Added install task/event types on the frontend and Rust backend.
- Added `install_tool` / `cancel_install` Tauri command skeletons that return immediately.
- Added `useInstallEvents` and basic App wiring for install status and lock-state UI.
- Kept V0.3 commit 1 strictly skeleton-only: no real Git/Node/Python installers, no config commands, no privilege flow.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 2 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
37 modules transformed
dist/ artifacts generated
built in 686ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 0.48s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
8 tests passed
0 failed
```

## Key Decisions

- `install_tool(toolId)` currently only reserves the install slot and returns `Result<(), String>`.
- `cancel_install()` currently only clears the slot and returns immediately; real process tree kill comes in commit 3.
- Install event types were added without simulating a fake installer in production code.
- Frontend now understands install lock-state and can disable other install buttons when one task enters `started/running`.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not skip real command validation even for skeleton commits.
  - Do not fake a long-running installer just to make the UI look complete.
  - Do not drift into config, privilege, or real installer work before the correct commit.
- Self-check result:
  - Skipped validation: no
  - Turned install into blocking invoke: no
  - Drifted outside commit 1 scope: no
  - Used fake production installer flow: no
  - Modified system proxy or npm global config: no
  - Added real Git/Node/Python installer execution: no

# V0.3 Commit 2 Validation

## Time

2026-05-02 17:31 (Asia/Shanghai)

## Scope

- Added `get_config()` / `update_config(patch)` / `reset_config()` with PRD-aligned naming.
- Added `is_admin()` real detection and `restart_as_admin()` basic entry command.
- Added frontend config/privilege wiring and displayed install network mode, npm registry, and non-admin hint.
- Added npm single-use registry helper without touching global npm config.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 3 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 719ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 1.64s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
14 tests passed
0 failed
```

## Key Decisions

- Config command names now follow the PRD exactly: `get_config`, `update_config`, `reset_config`.
- `restart_as_admin()` is implemented as a best-effort PowerShell `Start-Process -Verb RunAs` entry and is allowed to return a clear error if the relaunch request fails.
- npm registry support is currently helper-only and stays single-use; no `npm config set registry`.
- AppConfig is stored at `%APPDATA%\ai-coding-installer\config.json` and keeps V0.3 free of unused `pipIndexMode`.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not introduce a second config API naming scheme.
  - Do not let `restart_as_admin()` derail the mainline if Windows relaunch details are brittle.
  - Do not mutate system proxy or global npm config for convenience.
  - Do not skip real command validation just because this is still a skeleton stage.
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 2 scope: no
  - Added real installer execution: no
  - Introduced blocking install invoke: no
  - Modified system proxy: no
  - Executed `npm config set registry`: no

# V0.3 Commit 3 Validation

## Time

2026-05-02 23:58 (Asia/Shanghai)

## Scope

- Added a real background install runner skeleton that spawns a child process and returns from `install_tool()` immediately.
- Added install task PID tracking, cancel intent tracking, and lock release helpers in shared app state.
- Added Windows process tree kill helpers using `taskkill /F /T /PID`.
- Added line-by-line stdout/stderr redaction before progress emit and log writing.
- Added a controllable test installer path for architecture verification only.
- Kept V0.3 commit 3 strictly out of real Git / Node / Python installers.

## Commit

- commit hash: `5a26fb5`

## Difficulty Handling Record

### Current stage

V0.3 commit 3 verification

### Current task

Run the full acceptance commands for background install runner, lock, cancel, and redaction changes.

### Failed commands

- `npx vitest run src/App.test.tsx --reporter=verbose`
- `npm run build`

### Error summary

- Both commands timed out when I tried to run them in parallel with Cargo commands.
- Rust output showed `Blocking waiting for file lock on ...`, so the failure was command scheduling noise, not an implementation regression.

### superpower

- Available: no
- Note: `superpower 不可用。已改为 findskills + 仓库文档 + 保守实现方案。`

### findskills

- Available: yes
- Queries used:
  - `windows process tree`
  - `tauri event logging redaction`
- Relevant skills found:
  - `understanding-tauri-process-model`
  - `process-management`
  - `tauri-event-system`
  - `pii-redaction-logging-policy-builder`
- Adopted guidance:
  - avoid parallel commands that compete for build/package locks
  - keep PID tracking explicit
  - keep event flow and redaction order explicit

### pua reminders adopted

- Do not treat a file-lock timeout as proof the feature is broken.
- Do not skip re-running the same acceptance commands serially.
- Do not let command orchestration noise blur the plan-vs-implementation judgment.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 4 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 806ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 0.81s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
20 tests passed
0 failed
```

## Key Decisions

- `install_tool()` now remains non-blocking and only starts a background task.
- `cancel_install()` now uses Windows process tree kill semantics and treats “process already exited” as a cleanup path, not a hard failure.
- Redaction happens in Rust before `install:progress` emit and before log write.
- The runner currently supports controlled test commands only; real Git / Node / Python installers are intentionally deferred to commit 4.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not pretend test commands are real base dependency installers.
  - Do not block the Tauri invoke while the child process is running.
  - Do not skip the process tree requirement just because the child is only a validation runner.
  - Do not send raw sensitive output to the frontend or logs.
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 3 scope: no
  - Added real Git / Node / Python installers: no
  - Introduced blocking install invoke: no
  - Modified system proxy: no
  - Executed `npm config set registry`: no
  - Added one-click install: no
  - Stored/read/uploaded API keys: no

## TODO

- Commit 4 will replace the controlled test runner with real Git / Node / Python installers.
- Commit 4 will trigger post-install detection for the relevant tools.

# V0.3 Commit 4 Validation

## Time

2026-05-03 00:15 (Asia/Shanghai)

## Scope

- Replaced the commit 3 test-only install command mapping with real installer mappings for Git for Windows, Node.js LTS, and Python 3.11.
- Kept `install_tool()` non-blocking and reused the existing background runner, event chain, lock, cancellation, and redaction flow.
- Added manual-proxy winget warning suggestions to the installer start event.
- Added post-install re-detection targets for `git`, `node`, `npm`, and `python`.
- Kept V0.3 commit 4 strictly out of Claude / Codex / OpenCode / ccSwitch installation work.

## Difficulty Handling Record

### Current stage

V0.3 commit 4 verification

### Current task

Run the frontend acceptance commands after wiring real base dependency installers.

### Failed commands

- `npm run build` while run in parallel with `vitest`
- `npm run build` after several previous timed-out build attempts left background `node` processes alive

### Error summary

- `npx tsc --noEmit` passed, so TypeScript compilation itself was healthy.
- `vite build` timed out because old `npm/vite/vitest` child processes were still running in the background after earlier timeout-based interruptions.
- This was execution noise, not a code-level regression in the new installers.

### superpower

- Available: no
- Note: `superpower 不可用。已改为 findskills + 仓库文档 + 保守排查路径。`

### findskills

- Available: yes
- Queries used:
  - `winget windows install runner`
  - `tauri uac windows privilege`
  - `vite build timeout windows`
- Relevant skills found:
  - `process-management`
  - `understanding-tauri-process-model`
  - `tauri-event-system`
  - `vite`
- Adopted guidance:
  - isolate the hanging command from the rest of the build chain
  - inspect process state before blaming application code
  - clear only the repository-related residual `node` build processes

### pua reminders adopted

- Do not misdiagnose a stuck build process as a product-code failure.
- Do not keep retrying the same command without shrinking the problem.
- Do not skip rerunning the acceptance commands after cleaning residual processes.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 4 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 757ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 10.15s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
26 tests passed
0 failed
```

## Key Decisions

- Real base dependency installers now use `winget install` for:
  - `Git.Git`
  - `OpenJS.NodeJS.LTS`
  - `Python.Python.3.11`
- `install_tool()` still returns immediately and only starts the background task.
- Manual-proxy mode now adds the required winget limitation warning as an install-start suggestion instead of trying to mutate system proxy state.
- Successful installs now trigger backend re-detection for the relevant tools instead of making the frontend guess the new state.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not drift into AI-tool installers or one-click install.
  - Do not turn real installers back into blocking invoke calls.
  - Do not “solve” proxy friction by writing system proxy or global npm config.
  - Do not forget post-install re-detection.
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 4 scope: no
  - Added Claude / Codex / OpenCode / ccSwitch installers: no
  - Introduced blocking install invoke: no
  - Modified system proxy: no
  - Executed `npm config set registry`: no
  - Added one-click install: no
  - Modified system PATH automatically: no

# V0.4 Commit 1 Validation

## Time

2026-05-03 10:40 (Asia/Shanghai)

## Scope

- Added reusable AI npm package metadata and command construction helpers.
- Added single-use npm registry argument append helpers without touching global npm config.
- Added Node/npm prerequisite check helpers for future AI tool installers.
- Added reusable npm global command recheck helpers for Claude / Codex / OpenCode follow-up installs.
- Kept V0.4 commit 1 strictly out of real Claude / Codex / OpenCode installation and out of all ccSwitch work.

## Difficulty Handling Record

### Current stage

V0.4 commit 1 implementation and verification

### Current task

Wire AI npm helper infrastructure into the existing codebase without expanding the install surface.

### Failed commands

- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml installer::npm -- --nocapture`

### Error summary

- `installer/npm.rs` tried to import `detector::shared` and `detector::npm_global` directly.
- Both modules are private, so Rust failed with `module ... is private`.

### superpower

- Available: no
- Note: `superpower 不可用。已改为 findskills + 仓库文档 + 保守实现方案。`

### findskills

- Available: yes
- Relevant guidance reused:
  - keep internal module boundaries narrow
  - expose only minimal wrappers instead of making whole internal modules public

### pua reminders adopted

- Do not solve a helper-layer compile error by over-exposing detector internals.
- Keep the fix within commit 1 scope.
- Do not jump ahead to real Claude / Codex / OpenCode installers.

### Final fix

- Added minimal `pub(crate)` detector wrapper functions for:
  - Node runtime presence
  - npm global probe path
  - current npm status
  - AI npm command recheck
- Updated `installer/npm.rs` to depend on those wrappers instead of private modules.

## Additional Verification Noise Handling

### Failed commands

- `npx vitest run src/App.test.tsx --reporter=verbose`
- `npm run build`

### Error summary

- `vitest` worker startup timed out when the frontend commands were run in parallel.
- `npm run build` also timed out in the same parallel verification pass.
- Rust acceptance still passed, indicating execution-layer contention rather than a product-code regression.

### Fix

- Inspected active `node.exe` processes.
- Killed only repository-related stale `npm / vite / vitest` processes.
- Re-ran frontend test and build serially.

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 4 passed
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 727ms
```

### 3. Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 1.75s
```

### 4. Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
32 tests passed
0 failed
```

## Key Decisions

- npm helper data now covers:
  - `@anthropic-ai/claude-code`
  - `@openai/codex`
  - `opencode-ai`
- Registry handling remains single-use only and never writes `.npmrc` or global npm config.
- This commit only prepares AI npm install infrastructure; it does not expose new real installation behavior to users yet.

## PUA Phase Review

- `pua` available: yes
- Key reminders adopted:
  - Do not let helper work masquerade as real AI installer completion.
  - Do not write `npm config set registry`.
  - Do not expand into ccSwitch or concrete AI installers yet.
  - Do not skip full acceptance after compile-boundary fixes.
- Self-check result:
  - Skipped validation: no
  - Drifted outside commit 1 scope: no
  - Added real Claude / Codex / OpenCode installers: no
  - Introduced blocking install invoke: no
  - Modified system proxy: no
  - Executed `npm config set registry`: no
  - Added one-click install: no

# Code Review Stabilization Fix 1 Validation

## Time

2026-05-04 21:32 (Asia/Shanghai)

## Commit Goal

- Commit target: `fix: stabilize install status and cancellation flow`
- Phase: Code Review & Stabilization Phase
- PRD source: `D:\aicoding\ai_coding_环境助手_prd (5).md`
- PRD title/version: `AI Coding environment assistant PRD v2.2`

## Three-role Flow

- Plan Agent: real subagent used. Planned only P1-1 scope.
- Execute Agent: real subagent used. It initially drifted by running full-crate `cargo fmt`, causing unrelated formatting/stat noise. PUA check triggered; non-target formatting noise was removed and the implementation was manually narrowed back to the P1-1 scope.
- Verify Agent: real subagent used. It found frontend verification was blocked by PowerShell execution policy and sandbox `spawn EPERM`. Verification was rerun serially with approved elevated commands.

## PUA / findskills / superpower Record

- `pua` available: yes.
- PUA checkpoints used:
  - commit start
  - implementation drift
  - validation failure
  - before commit readiness review
- `findskills` available: yes. Used as process guidance after repeated frontend build/test command failures.
- `superpower` tool unavailable.
- Note: superpower unavailable. Already switched to findskills + repository docs + conservative implementation.

## Scope

- `src-tauri/src/commands/install.rs`
- `src-tauri/src/installer/runner.rs`
- `src-tauri/src/process/kill_tree.rs`
- `src-tauri/src/state.rs`

## Changes

- `cancel_install` now records cancel intent before attempting to kill the process tree.
- The runner emits `install:status` with `phase=running` after the process starts.
- Final status selection now prefers `cancelled` when cancel intent races with a natural process exit.
- Successful, failed, cancelled, and timeout final paths still clear the install lock before final status emission.
- `taskkill` process-not-found output is treated as `AlreadyExited`, including common English and Chinese output patterns.
- Added focused Rust tests for running status, final-state selection, cancel intent preservation, and taskkill process-not-found handling.

## Validation Commands

### Frontend test

Command:

```powershell
cmd.exe /c npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files 1 passed
Tests 9 passed
```

### Frontend build

Command:

```powershell
cmd.exe /c npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist assets generated
built in 813ms
```

### Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile target(s) in 1.98s
```

### Rust tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
57 tests passed
0 failed
```

## Validation Noise Handling

- `npx vitest` through PowerShell failed because `npx.ps1` is blocked by execution policy. The same required command was run through `cmd.exe /c npx ...`.
- Initial sandboxed Vitest/Vite runs failed with `spawn EPERM`; approved elevated reruns passed.
- Two `npm run build` attempts timed out due stale `node.exe` processes from earlier failed verification. Only verification-spawned node processes were stopped, then the fixed build command passed.
- `git status --short` may show line-ending/stat noise for some Rust files, but `git diff --name-only` confirms actual content changes are limited to the four scope files above.

## PRD Constraint Check

- Skipped validation: no.
- Used mock instead of real logic: no.
- Drifted outside P1-1 final scope: no.
- Converted install invoke into long-running blocking invoke: no.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- Added automatic outdated detection: no.
- Removed PRD core functionality to pass build: no.

## Commit Result

- Commit hash: `0a3f506`
- Commit message: `fix: stabilize install status and cancellation flow`
- Post-commit log:

```text
0a3f506 fix: stabilize install status and cancellation flow
4b1e93e feat: add reinstall and latest install actions
fe0652f feat: add ccswitch download source support
794ec14 feat: add ccswitch path and launch support
f9531d9 feat: implement claude codex opencode installers
```

# P1-2 Execute Agent Implementation

## Time

2026-05-04 21:41 (Asia/Shanghai)

## Commit Goal

- Commit target: `fix: expand log redaction coverage`
- Phase: P1-2 Execute Agent implementation

## PUA Start Check

- `pua` available: yes.
- Start reminders adopted:
  - keep writes limited to `src-tauri/src/logger/redact.rs` and `docs/dev-log.md`
  - use TDD: add failing Rust tests before implementation
  - do not run `cargo fmt` or whole-repo formatting
  - do not change install status flow, AppConfig, ccSwitch, frontend, or V0.5 features
  - do not stage or commit

## Scope

- `src-tauri/src/logger/redact.rs`
- `docs/dev-log.md`

## Changes

- Expanded `redact_line()` coverage for named API-key environment variables:
  - `OPENAI_API_KEY=...`
  - `ANTHROPIC_API_KEY=...`
  - `GEMINI_API_KEY=...`
  - `API_KEY=...`
- Preserved existing redaction for:
  - `Authorization: Bearer ...`
  - `sk-...`
  - `token=...`
  - `api_key=...`
- Added proxy URL credential redaction for:
  - `http://user:pass@...`
  - `https://user:pass@...`
  - `socks5://user:pass@...`
- Added focused Rust tests asserting sensitive originals do not appear in `redact_line()` output.

## TDD Record

- Red test command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml logger::redact -- --nocapture
```

- Red result:

```text
2 passed; 3 failed
Failures showed OPENAI/ANTHROPIC/GEMINI env values and proxy credentials leaked.
```

- Green test command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml logger::redact -- --nocapture
```

- Green result:

```text
5 passed; 0 failed
```

## Constraint Check

- Skipped TDD: no.
- Ran `cargo fmt`: no.
- Modified install status machine: no.
- Modified AppConfig: no.
- Modified ccSwitch: no.
- Modified frontend: no.
- Added V0.5 feature work: no.
- Staged or committed: no.

## PUA Implementation Complete Check

- Node: after P1-2 implementation.
- Skipped validation: no; focused Rust redaction tests and full Rust tests were run by Execute Agent.
- Used mock instead of real logic: no; `redact_line()` rules were expanded directly.
- Drifted from PRD: no.
- Converted install command into blocking invoke: no.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- Added automatic outdated detection: no.
- Scope too large: no; diff is limited to `src-tauri/src/logger/redact.rs` and `docs/dev-log.md`.
- Commit readiness: pending Verify Agent full acceptance commands and `git diff` review.

# P1-2 Verify Agent Validation

## Time

2026-05-04 21:50 (Asia/Shanghai)

## Verify Agent Result

- Verify Agent checked the plan scope, `git diff --name-only`, `src-tauri/src/logger/redact.rs`, `docs/dev-log.md`, and the `runner.rs` stdout/stderr redaction call chain.
- Verify Agent confirmed the runtime call chain still reads each stdout/stderr line, runs `redact_line(&line)`, emits `install:progress` with the redacted line, and writes the same redacted line to the log file.
- Verify Agent attempted `cmd.exe /c npx vitest run src/App.test.tsx --reporter=verbose` inside the sandbox.
- Sandbox result: failed before tests with Vite/esbuild `spawn EPERM`.
- PUA failure check: do not treat sandbox process-spawn failure as an app regression; rerun the same required command with approved elevated execution.
- findskills invoked: yes, used to confirm fallback skill workflow.
- superpower unavailable. Already switched to findskills + repository docs + conservative implementation plan.

## Main Process Completion Of Fixed Validation

### Frontend test

Command:

```powershell
cmd.exe /c npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed with elevated execution after sandbox `spawn EPERM`.

Summary:

```text
Test Files 1 passed
Tests 9 passed
```

### Frontend build

Command:

```powershell
cmd.exe /c npm run build
```

Result:

Passed with elevated execution.

Summary:

```text
38 modules transformed
built in 801ms
```

### Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile target(s) in 1.19s
```

### Targeted Rust redaction tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml logger::redact
```

Result:

Passed.

Summary:

```text
5 passed
0 failed
```

### Rust full test suite

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
60 passed
0 failed
```

## PUA Pre-Commit Check

- Node: before commit.
- Skipped validation: no.
- Used mock instead of real logic: no.
- Drifted outside P1-2 scope: no.
- Converted install command into blocking invoke: no.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- Added automatic outdated detection: no.
- Removed PRD core functionality to pass build: no.
- Commit before running acceptance commands: no.
- Commit should include untracked PRD file: no.

## Commit Result

- Commit hash: `d37b6fe`
- Commit message: `fix: expand log redaction coverage`
- Post-commit log:

```text
d37b6fe fix: expand log redaction coverage
0a3f506 fix: stabilize install status and cancellation flow
4b1e93e feat: add reinstall and latest install actions
fe0652f feat: add ccswitch download source support
794ec14 feat: add ccswitch path and launch support
```

# P1-3 Execute Agent Implementation

## Time

2026-05-04 21:59 (Asia/Shanghai)

## Commit Goal

- Commit target: `fix: harden app config recovery and validation`
- Phase: P1-3 Execute Agent implementation

## PUA Start Check

- `pua` available: not re-read this turn; process reminders carried from Plan/Execute instruction set and checked before edits.
- Start reminders adopted:
  - keep writes limited to `src-tauri/src/config.rs` and `docs/dev-log.md`
  - use TDD with path-based helpers to avoid real `%APPDATA%`
  - do not run `cargo fmt` or whole-repo formatting
  - do not change log redaction, install status flow, ccSwitch `direct_zip`, frontend, or V0.5 features
  - do not stage or commit

## Scope

- `src-tauri/src/config.rs`
- `docs/dev-log.md`

## Changes

- Added path-based config helpers for tests and shared implementation:
  - `get_config_from_path(path)`
  - `write_config_to_path(path, config)`
  - `update_config_at_path(path, patch)`
- `get_config()` still resolves `%APPDATA%\ai-coding-installer\config.json`.
- Malformed `config.json` now recovers by:
  - copying the corrupt file to `config.bak.YYYYMMDD-HHMMSS.json` in the same directory
  - writing default config back to `config.json`
  - returning default config instead of failing the command
- Existing valid config is sanitized and written back, which removes unknown JSON fields such as API key, Provider, and `pipIndexMode`.
- `customNpmRegistry` is trimmed and validated when present; `npmRegistry=custom` requires a valid custom URL.
- ccSwitch download source URLs are trimmed and validated when source list is provided; an empty default list remains valid.
- URL validation is conservative and offline-only:
  - allowed schemes: `http://`, `https://`
  - host must be non-empty
  - userinfo credentials are rejected

## TDD Record

- Red test command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml config::tests -- --nocapture
```

- Red result:

```text
Compilation failed because get_config_from_path, write_config_to_path, and update_config_at_path did not exist yet.
```

- Green test command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml config::tests -- --nocapture
```

- Green result:

```text
8 passed; 0 failed
```

## Constraint Check

- Skipped TDD: no.
- Ran `cargo fmt`: no.
- Modified log redaction: no.
- Modified install status machine: no.
- Modified ccSwitch `direct_zip`: no.
- Modified frontend: no.
- Added V0.5 feature work: no.
- Added/saved API Key, Provider, or `pipIndexMode`: no.
- Staged or committed: no.

## Main Flow Review Adjustment

- Tightened URL validation to reject whitespace anywhere in the URL, not only in the authority/host section.
- Tightened backup test coverage to assert the exact `config.bak.YYYYMMDD-HHMMSS.json` filename shape.
- Scope remains limited to `src-tauri/src/config.rs` and `docs/dev-log.md`.

## PUA Implementation Complete Check

- Node: after P1-3 implementation and main-flow review adjustment.
- Skipped validation: no; focused config tests and full Rust tests were run by Execute Agent, and full fixed validation is still pending Verify Agent.
- Used mock instead of real logic: no; config recovery and validation are implemented in Rust config loading/update paths.
- Drifted from PRD: no.
- Saved/read/uploaded API Key: no new AppConfig fields; unknown `apiKey`/Provider/`pipIndexMode` JSON is normalized out.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- Added automatic outdated detection: no.
- Scope too large: no; diff is limited to `src-tauri/src/config.rs` and `docs/dev-log.md`.
- Commit readiness: pending Verify Agent full acceptance commands and `git diff` review.

# P1-3 Validation

## Time

2026-05-04 22:02 (Asia/Shanghai)

## Main Process Validation

### Targeted Rust config tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml config::tests
```

Result:

Passed.

Summary:

```text
9 passed
0 failed
```

### Frontend test

Command:

```powershell
cmd.exe /c npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed with elevated execution because sandboxed Vite/esbuild can fail with `spawn EPERM`.

Summary:

```text
Test Files 1 passed
Tests 9 passed
```

### Frontend build

Command:

```powershell
cmd.exe /c npm run build
```

Result:

Passed with elevated execution.

Summary:

```text
38 modules transformed
built in 763ms
```

### Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile target(s) in 1.31s
```

### Rust full test suite

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
66 passed
0 failed
```

## PUA Pre-Commit Check

- Node: before P1-3 commit.
- Skipped validation: no.
- Used mock instead of real logic: no.
- Drifted outside P1-3 scope: no.
- Converted install command into blocking invoke: no.
- Saved/read/uploaded API Key: no new config fields; unknown API key-like fields are normalized out.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- Added automatic outdated detection: no.
- Removed PRD core functionality to pass build: no.
- Commit before running acceptance commands: no.
- Commit should include untracked PRD file: no.

## Commit Result

- Commit hash: `76cc2ba`
- Commit message: `fix: harden app config recovery and validation`
- Post-commit log:

```text
76cc2ba fix: harden app config recovery and validation
d37b6fe fix: expand log redaction coverage
0a3f506 fix: stabilize install status and cancellation flow
4b1e93e feat: add reinstall and latest install actions
fe0652f feat: add ccswitch download source support
```

# P1-4 Execute Agent Implementation

## Time

2026-05-04 22:12 (Asia/Shanghai)

## Commit Goal

- Commit target: `fix: align ccswitch download source behavior`
- Phase: P1-4 Execute Agent implementation
- Selected option: B, disable `direct_zip` pseudo support and keep only implemented `direct_exe` behavior.

## PUA Start Check

- `pua` invoked by Plan/Execute workflow before implementation.
- Start reminders adopted:
  - do not leave a schema that claims `direct_zip` is supported when installer behavior does not implement zip extraction
  - do not add unknown third-party download sources
  - keep default ccSwitch download sources empty
  - do not read/write ccSwitch DB
  - do not write Provider config
  - do not save/read/upload API Key
  - keep writes limited to P1-4 scope files
  - do not stage or commit

## Scope

- `src-tauri/src/installer/ccswitch.rs`
- `src-tauri/src/config.rs`
- `src/types/config.ts`
- `docs/dev-log.md`

## Changes

- `ordered_enabled_sources()` now filters to enabled, non-empty, `DirectExe` sources only.
- Generated PowerShell download script now downloads `.exe` only and no longer emits `$source.Kind`, `.zip`, or `direct_zip` branches.
- Single-source failures now use `Write-Output` instead of `Write-Error`, so `$ErrorActionPreference = 'Stop'` does not stop the retry loop before the next source.
- All sources empty, filtered, or failed still results in the manual ccSwitch path fallback message.
- Frontend `CcSwitchDownloadSourceKind` type now exposes only `"direct_exe"`.
- Config sanitization filters historical `DirectZip` sources before validation/write-back, so old `direct_zip` entries are not persisted.
- No default download source was added.

## Validation

### Targeted ccSwitch tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml ccswitch
```

Result:

Passed.

Summary:

```text
13 passed
0 failed
```

### Targeted config tests

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml config
```

Result:

Passed.

Summary:

```text
13 passed
0 failed
```

### Frontend test

Command:

```powershell
cmd.exe /c npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed with elevated execution because sandboxed Vite/esbuild can fail with `spawn EPERM`.

Summary:

```text
Test Files 1 passed
Tests 9 passed
```

### Frontend build

Command:

```powershell
cmd.exe /c npm run build
```

Result:

Passed with elevated execution.

Summary:

```text
38 modules transformed
built in 765ms
```

### Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile target(s) in 1.12s
```

### Rust full test suite

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
71 passed
0 failed
```

## PUA Pre-Commit Check

- Node: before P1-4 commit.
- Skipped validation: no.
- Used mock instead of real logic: no.
- Left `direct_zip` pseudo support active: no.
- Added unknown third-party GitHub accelerator/default source: no.
- Default source can be empty: yes.
- Manual path fallback after no usable sources/all source failures: yes.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- Added automatic outdated detection: no.
- Removed PRD core functionality to pass build: no.
- Commit should include untracked PRD file: no.

# P1-4 Execute Agent Implementation

## Time

2026-05-04 22:10 (Asia/Shanghai)

## Commit Goal

- Commit target: `fix: align ccswitch download source behavior`
- Selected plan option: B, disable/remove `direct_zip` pseudo-support and keep only real `direct_exe`.

## PUA Start Check

- Start reminders adopted:
  - keep writes limited to `src-tauri/src/installer/ccswitch.rs`, `src-tauri/src/config.rs`, `src/types/config.ts`, and `docs/dev-log.md`
  - use TDD before implementation
  - do not run `cargo fmt` or whole-repo formatting
  - do not change log redaction, install status flow, AppConfig recovery logic itself, frontend V0.5 features, Provider, API keys, or ccSwitch DB behavior
  - do not stage or commit

## Scope

- `src-tauri/src/installer/ccswitch.rs`
- `src-tauri/src/config.rs`
- `src/types/config.ts`
- `docs/dev-log.md`

## Changes

- `ordered_enabled_sources` now returns only enabled, non-blank URL, `DirectExe` sources.
- ccSwitch download script now always downloads `.exe` files and no longer contains `$source.Kind`, `.zip`, `direct_zip`, or unsupported zip branches.
- Per-source failure logging now uses non-terminating `Write-Output` inside `catch`, so `$ErrorActionPreference = 'Stop'` does not stop the loop before the next source.
- If all sources are empty, filtered out, or fail, the installer still throws the existing manual ccSwitch path fallback message.
- `sanitize_config` filters legacy `DirectZip` sources so old configs or patches do not keep writing `direct_zip` back to disk.
- Frontend config type now exposes `CcSwitchDownloadSourceKind` as `"direct_exe"` only.

## TDD Record

- Red test command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml ccswitch -- --nocapture
```

- Red result:

```text
9 passed; 4 failed
Failures showed direct_zip still entered source ordering, command_spec, script branches, and Write-Error failure logging.
```

- Additional red test command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml sanitize_filters_direct_zip -- --nocapture
```

- Additional red result:

```text
0 passed; 1 failed
Failure showed sanitize_config still wrote direct_zip sources.
```

- Green test commands:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml ccswitch -- --nocapture

$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml sanitize_filters_direct_zip -- --nocapture
```

- Green results:

```text
ccswitch filter: 13 passed; 0 failed
sanitize filter: 1 passed; 0 failed
```

## Constraint Check

- Skipped TDD: no.
- Ran `cargo fmt`: no.
- Modified log redaction: no.
- Modified install status machine: no.
- Modified AppConfig damage recovery logic itself: no.
- Added default download source: no.
- Used unknown third-party accelerator: no.
- Read/wrote ccSwitch DB: no.
- Wrote Provider: no.
- Saved API Key: no.
- Added V0.5 feature work: no.
- Staged or committed: no.

## Commit Result

- Commit hash: `ab3c2b4`
- Commit message: `fix: align ccswitch download source behavior`
- Post-commit log:

```text
ab3c2b4 fix: align ccswitch download source behavior
76cc2ba fix: harden app config recovery and validation
d37b6fe fix: expand log redaction coverage
0a3f506 fix: stabilize install status and cancellation flow
4b1e93e feat: add reinstall and latest install actions
```

# P1 Stabilization Phase Complete

## PUA Stage Complete Check

- Node: after all P1 fixes.
- P1-1 install status/cancellation committed: `0a3f506`.
- P1-2 log redaction committed: `d37b6fe`.
- P1-3 AppConfig recovery/validation committed: `76cc2ba`.
- P1-4 ccSwitch source behavior committed: `ab3c2b4`.
- Skipped validation: no.
- Used mock instead of real logic: no.
- Drifted outside PRD: no.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- Added automatic outdated detection: no.
- Removed PRD core functionality to pass build: no.
- Next phase: continue V0.5 Commit 2, PATH repair instructions UI only.

# V0.5 Commit 2 Execute - PATH Repair Instructions

## Start Check

- Time: 2026-05-04 22:24.
- Scope: only `src/App.tsx`, `src/App.css`, `src/App.test.tsx`, `docs/dev-log.md`.
- PUA check: no passive waiting; use source inspection plus TDD red/green before claiming done.
- Guardrails: no Rust command, no `repair_tool(mode)`, no automatic PATH mutation, no `setx`, no PowerShell execution for repair, no staging/commit.

## TDD

- Red test added first in `src/App.test.tsx`: an `installed_but_path_missing` tool should expose a PATH repair instructions entry, open a modal, and copy a manual user PATH command through `navigator.clipboard.writeText`.
- Initial red result: `PATH repair instructions` button was absent.
- Implementation added the minimal UI state, modal, command generation, and CSS needed for the test.

## Implementation Notes

- `src/App.tsx` now shows the PATH repair entry only for tools whose status is `installed_but_path_missing` and have an executable path.
- The modal displays tool name, executable path, the directory to add to PATH, manual steps, and an explicit note that the client will not automatically modify PATH.
- The copied command uses `[Environment]::SetEnvironmentVariable(..., "User")`, reads the user PATH, and appends the directory only when it is not already present.
- The copy action only calls `navigator.clipboard.writeText(...)`; no command execution path was added.

## Validation

- `cmd.exe /c npx vitest run src/App.test.tsx --reporter=verbose`: passed, 10 tests.
- `cmd.exe /c npx tsc --noEmit`: passed with exit code 0; PowerShell emitted an execution-policy warning while loading the user profile.
- `git diff --name-only`: only allowed files changed.
- `git status --short`: allowed modified files plus the existing untracked PRD file.

## Constraint Check

- Modified files outside allowed scope: no.
- Ran `cargo fmt` or whole-repo formatting: no.
- Added Rust command or `repair_tool(mode)`: no.
- Automatically modified PATH or executed repair command: no.
- Used `setx`: no.
- Implemented V0.5 Commit 3/4: no.
- Staged or committed: no.

# V0.5 Commit 2 Validation

## Time

2026-05-04 22:25 (Asia/Shanghai)

## Main Process Validation

### Frontend test

Command:

```powershell
cmd.exe /c npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed with elevated execution because sandboxed Vite/esbuild can fail with `spawn EPERM`.

Summary:

```text
Test Files 1 passed
Tests 10 passed
```

### Frontend build

Command:

```powershell
cmd.exe /c npm run build
```

Result:

Passed with elevated execution.

Summary:

```text
38 modules transformed
built in 781ms
```

### Rust / Tauri check

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile target(s) in 0.53s
```

### Rust full test suite

Command:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
71 passed
0 failed
```

### Constraint grep

Command:

```powershell
rg -n "repair_tool|setx|netsh winhttp|npm config set" src src-tauri docs/dev-log.md
```

Result:

Passed. No new runtime repair command, no `setx`, no `netsh winhttp`, and no `npm config set` path was added. Matches are historical docs/tests or expected negative checks.

## PUA Pre-Commit Check

- Node: before V0.5 Commit 2 commit.
- Skipped validation: no.
- Used mock instead of real logic: no; UI copies a concrete user PATH command to clipboard.
- Added Rust command or `repair_tool(mode)`: no.
- Automatically modified PATH: no.
- Executed the copied PowerShell command: no.
- Used `setx`: no.
- Took over system proxy: no.
- Added one-click install: no.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Implemented V0.5 Commit 3/4 early: no.
- Commit should include untracked PRD file: no.

## Resume Validation

- Time: 2026-05-05 12:21 (Asia/Shanghai).
- Resume note: user asked to continue without restarting. Git history shows P1-2/P1-3/P1-4 already committed; current dirty worktree belongs to V0.5 Commit 2, not P1-2.
- `npx vitest run src/App.test.tsx --reporter=verbose`: passed, 10 tests.
- `npm run build`: passed, 38 modules transformed, built in 733ms.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 71 tests.
- Verify Agent: allowed commit with message `feat: add path repair instructions`.
- PRD input file remains untracked and must be excluded from commit.

## Commit Result

- Commit hash: `acc1dd5`
- Commit message: `feat: add path repair instructions`
- Post-commit log:

```text
acc1dd5 feat: add path repair instructions
ab3c2b4 fix: align ccswitch download source behavior
76cc2ba fix: harden app config recovery and validation
d37b6fe fix: expand log redaction coverage
0a3f506 fix: stabilize install status and cancellation flow
4b1e93e feat: add reinstall and latest install actions
fe0652f feat: add ccswitch download source support
794ec14 feat: add ccswitch path and launch support
```

# V0.5 Commit 3 Execute - Subscription Entry And Quick Actions

## Resume And Role Setup

- Time: 2026-05-05 12:35 (Asia/Shanghai).
- Current commit target: `feat: add subscription entry and quick actions`.
- Plan Agent: real subagent started, scoped only to V0.5 Commit 3.
- Execute Agent: real subagent started, scoped to frontend QuickActions/config UI/tests only.
- Verify Agent: real subagent started, read-only validation scope.
- PUA node: commit start. Checked for passive waiting, skipped validation, PRD drift, API Key storage, Provider writes, system proxy takeover, ccSwitch DB reads/writes, node client work, subscription parsing, one-click install, automatic outdated, and oversize scope.
- PRD input file remains untracked and must be excluded from commit.

## Implementation Notes

- Kept `subscriptionPageUrl` AppConfig default empty.
- Added `open_subscription_page` as a browser handoff only; it does not download, parse, or set proxy.
- Added frontend API/QuickActions for detect all, open ccSwitch, open subscription page, open install network settings, and view logs area.
- Disabled quick ccSwitch open when no detected executable path exists, with title `请先安装或指定 ccSwitch 路径`.
- Added a subscription page URL input that saves through `update_config`.

## Validation

- Verify Agent first reported the frontend QuickActions/config UI gap; implementation was not allowed to commit until that gap was closed.
- PUA node: after implementation and after the first failed validation. The initial frontend run exposed JSX/text issues; the first Rust test run exposed missing `subscription_page_url` fields in installer test fixtures. Both were fixed in scope.
- `npx vitest run src/App.test.tsx --reporter=verbose`: passed, 13 tests.
- `npm run build`: passed, 38 modules transformed.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 75 tests.

## Constraint Check

- PUA node: before commit. Diff was reviewed after validation; commit scope remains V0.5 Commit 3 only.
- Skipped validation: no.
- Used mock instead of real logic: no; QuickActions call the real Tauri API wrapper and Rust opens the default browser from AppConfig.
- Parsed/downloaded subscription content: no.
- Started proxy or took over system proxy: no.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Read/wrote ccSwitch database: no.
- Added one-click install or automatic outdated detection: no.
- Implemented log zip export or V1.0 packaging: no.
- PRD input file remains untracked and must be excluded from commit.

## Commit Result

- Commit hash: `ad77f24`
- Commit message: `feat: add subscription entry and quick actions`
- Post-commit log:

```text
ad77f24 feat: add subscription entry and quick actions
acc1dd5 feat: add path repair instructions
ab3c2b4 fix: align ccswitch download source behavior
76cc2ba fix: harden app config recovery and validation
d37b6fe fix: expand log redaction coverage
0a3f506 fix: stabilize install status and cancellation flow
4b1e93e feat: add reinstall and latest install actions
fe0652f feat: add ccswitch download source support
```

# V0.5 Commit 4 Execute - Diagnostics Log Export

## Resume And Role Setup

- Time: 2026-05-05 22:17 (Asia/Shanghai).
- Current commit target: `feat: add diagnostics log export`.
- PRD input file remains untracked and must be excluded from commit.

## Implementation Notes

- Added `src-tauri/src/commands/logs.rs` with four Tauri commands: `get_log_preview`, `export_logs`, `open_full_log_file`, `open_log_directory`.
- `get_log_preview`: reads `.log` files from `%APPDATA%\ai-coding-installer\logs`, returns last N lines (default 5000) with VecDeque window, plus total count and truncation flag.
- `export_logs`: reads all `.log` files, writes a ZIP to `%APPDATA%\ai-coding-installer\exports\diagnostics-YYYYMMDD-HHMMSS.zip` with a `summary.txt` manifest; uses hand-written minimal ZIP (no compression, CRC32 only) to avoid adding a new crate dependency.
- `open_full_log_file` / `open_log_directory`: shell-open via `tauri_plugin_opener`.
- Registered all four commands in `src-tauri/src/lib.rs` invoke_handler and `pub mod logs` in `src-tauri/src/commands/mod.rs`.
- Frontend `src/lib/api.ts`: added `getLogPreview`, `openFullLogFile`, `openLogDirectory`, `exportDiagnosticsLogZip` wrappers.
- Frontend `src/App.tsx`: added Logs panel with log viewer (5000-line cap via `.slice(-LOG_VIEWER_LINE_LIMIT)`), "Open full log file", "Open log directory", "Export diagnostics zip" buttons wired to API calls.
- Frontend `src/App.css`: added `.log-toolbar`, `.log-count`, `.log-viewer`, `.log-line`, `.log-empty` styles.
- Removed unused `Read` import from `logs.rs` to eliminate compiler warning.

## Validation

- `npx vitest run src/App.test.tsx --reporter=verbose`: passed, 14 tests.
- `npm run build`: passed, 38 modules transformed.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed, 0 warnings.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 77 tests.

## Constraint Check

- Skipped validation: no.
- Used mock instead of real logic: no; ZIP write is real in Rust, frontend calls real Tauri invoke wrappers.
- Uploaded logs: no; export writes local ZIP only.
- Contains unredacted secrets: no; logs are already redacted by the backend logger before writing to `.log` files.
- Saved/read/uploaded API Key: no.
- Modified Provider configuration: no.
- Took over system proxy: no.
- Read/wrote ccSwitch database: no.
- Added one-click install: no.
- PRD input file remains untracked and must be excluded from commit.

# V0.5 Commit 5 Fix - Prevent Event Listener Leak Under React StrictMode

## Problem

Code review found `useDetectEvents.ts` and `useInstallEvents.ts` both use `listen()` which is async and returns an unlisten function. In React StrictMode or rapid unmount, the cleanup may run before `listen()` resolves, causing the listener to leak (unlisten is never called) or duplicate subscriptions.

## Fix

- `useDetectEvents.ts`: added `cancelled` flag. In the `.then()` callback, if `cancelled` is already true, call `dispose()` immediately and return. Cleanup sets `cancelled = true` then calls `unlisten?.()`.
- `useInstallEvents.ts`: same pattern applied to both `install:progress` and `install:status` listeners independently. Each has its own `cancelled` check in its `.then()` callback.

## Validation

- `npx vitest run src/App.test.tsx --reporter=verbose`: passed, 14 tests (28 total including worktree copy).
- `npm run build`: passed, 38 modules transformed.
- No Rust changes made, cargo check/test not required.

# Bugfix: resolve cancel-vs-success race in install runner

## Time

2026-05-05 (Asia/Shanghai)

## Commit Goal

`fix: resolve cancel-vs-success race in install runner`

## Scope

- `cancel_install` command now treats "no running install task" as a no-op (returns Ok) instead of propagating an error to the frontend. This handles the race where the install task finishes between the UI click and the command reaching the backend.
- `cancel_install` now uses best-effort kill: both `AlreadyExited` and unexpected taskkill errors are silently accepted. The `cancel_requested` flag drives the final Cancelled status regardless.
- Runner poll loop: changed `Err(error) => return Err(error)` in the kill branch to `Err(_) => {}`. A taskkill error while cancel is requested no longer causes a premature `Failed` event; the loop continues to observe process exit naturally and `cancel_requested` yields Cancelled.
- `mark_cancel_requested` on cleared state now has a documented test confirming it returns a benign "no running install task" error (not a panic or silent failure).

## Modified Files

- `src-tauri/src/commands/install.rs`
- `src-tauri/src/installer/runner.rs`
- `src-tauri/src/state.rs`

## Validation

### cargo check

```
Finished dev profile [unoptimized + debuginfo] target(s) in 2.75s
```

### cargo test

```
80 tests passed, 0 failed
```

## PRD Compliance

- install_tool still returns immediately (non-blocking spawn).
- cancel_install uses taskkill /F /T /PID (process tree, not just parent).
- No API key saved, no system proxy modified, no one-click install.

## PUA Self-Check

- Skipped validation: no
- Drifted outside commit scope: no
- Blocked install command: no
- Saved/read/uploaded API key: no
- Took over system proxy: no
- Made cancel blocking: no

# Bugfix: add proxy URL validation to AppConfig

## Time

2026-05-05 (Asia/Shanghai)

## Commit Goal

`fix: add proxy url validation to app config`

## Scope

- Added `validate_proxy_url()` that accepts `http://`, `https://`, `socks5://` and rejects all other schemes, credentials in URL, missing host, whitespace.
- `validate_config()` now enforces: when `mode == ManualProxy`, `proxy_url` must be present and valid. A non-None proxy_url in any mode is also validated for format.
- Added 7 new tests covering valid schemes, rejected schemes (ftp, socks4, bare host:port), credentials rejection, missing host, manual proxy mode without URL, and full integration test via `update_config_at_path`.

## Modified Files

- `src-tauri/src/config.rs`

## Validation

### cargo check

```
Finished dev profile [unoptimized + debuginfo] target(s) in 1.42s
```

### cargo test

```
87 tests passed, 0 failed
```

## PRD Compliance

- No API key saved or read.
- No system proxy taken over.
- No `netsh winhttp set proxy` executed.
- No `npm config set registry` executed.
- `pipIndexMode` not introduced.

## PUA Self-Check

- Skipped validation: no
- Drifted outside commit scope: no
- Took over system proxy: no
- Saved/read/uploaded API key: no

# Docs: clarify system proxy mode and set_tool_path behavior

## Time

2026-05-05 (Asia/Shanghai)

## Commit Goal

`docs: clarify system proxy mode and set_tool_path behavior`

## Scope

Updated README.md to document two known deviations from the PRD interface specification:

1. **SystemProxy mode limitations**: The client does not set or intercept the system proxy; it relies on inherited environment variables. winget does not read proxy env vars; UI warns when manual_proxy is active. `netsh winhttp set proxy` is never executed.

2. **`set_tool_path` command**: The PRD lists `set_tool_path(toolId, path)` as a planned Tauri command. Current implementation uses `update_config({ ccswitchPath })` instead. Functionality is equivalent; dedicated command may be added for strict PRD alignment in a future commit.

Also added a basic project README with development commands, architecture overview, and the V1.0 exclusions list.

## Modified Files

- `README.md`

## Validation

- `git status`: only README.md changed
- No code changes; cargo check/test not required

## PRD Compliance

- Documents deviations honestly without claiming unimplemented APIs exist
- No code changes; no new functionality introduced
- Does not falsely declare set_tool_path implemented

# Release Readiness Validation before V1.0

## Baseline

- Latest commit: `57b5f7c docs: clarify system proxy mode and set_tool_path behavior`
- Working tree: clean (only untracked `.claude/` and PRD input file present)

## Validation Commands

### 1. Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files  2 passed (2)
Tests  28 passed (28)
Duration  5.13s
```

### 2. Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
tsc && vite build
38 modules transformed
dist/ artifacts generated
built in 871ms
```

### 3. Rust / Tauri check

Command:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 0.95s
```

### 4. Rust tests

Command:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
87 tests passed
0 failed
```

## Result

- Pass
- Notes: All four validation commands passed. 87 Rust tests, 28 frontend tests, frontend build clean, cargo check clean. Working tree is clean. Ready to proceed to V1.0.

## PUA Self-Check

- Skipped validation: no
- Made up unimplemented API: no
- Changed code in documentation commit: no

# V1 Readiness Polish

## Time

2026-05-06 (Asia/Shanghai)

## Commit Goal

`fix: polish v1 readiness issues`

## Scope

1. **UI button text Chinese localization**: changed `Reinstall` → `重试安装`, `Install latest` → `安装最新版` in `src/App.tsx` and corresponding test in `src/App.test.tsx`.
2. **Exit original instance after restart_as_admin**: modified `restart_as_admin` in `src-tauri/src/commands/privilege.rs` to accept `tauri::AppHandle` and call `app.exit(0)` after successful `Start-Process -Verb RunAs`. No new dependency added; `app.exit(0)` is part of the existing `tauri` crate API.
3. **README ccSwitch download source limitation**: added "ccSwitch Download Sources" section to `README.md` documenting DirectExe-only support, no DirectZip, and empty default source list.

## Validation Commands

### Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files  2 passed (2)
Tests  28 passed (28)
```

### Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 1050ms
```

### Rust / Tauri check

Command:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 2.17s
```

### Rust tests

Command:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
87 tests passed
0 failed
```

## PRD Compliance

- No API key saved, read, or uploaded.
- No system proxy modified.
- No `netsh winhttp set proxy` executed.
- No `npm config set registry` executed.
- No automatic PATH modification.
- No new dependency added to Cargo.toml or package.json.

## PUA Self-Check

- Skipped validation: no
- Drifted outside scope: no
- Added new dependency: no
- Saved/read/uploaded API Key: no
- Took over system proxy: no

# V1.0 Windows Packaging Validation

## Time

2026-05-06 (Asia/Shanghai)

## Baseline

- Latest commit: `3e77adb fix: polish v1 readiness issues`
- Working tree: clean (only untracked `.claude/` and PRD input file present)

## Pre-Package Validation

### Frontend test

Command:

```powershell
npx vitest run src/App.test.tsx --reporter=verbose
```

Result:

Passed.

Summary:

```text
Test Files  1 passed (1)
Tests  14 passed (14)
```

### Frontend build

Command:

```powershell
npm run build
```

Result:

Passed.

Summary:

```text
38 modules transformed
dist/ artifacts generated
built in 807ms
```

### Rust / Tauri check

Command:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
Finished dev profile [unoptimized + debuginfo] target(s) in 0.67s
```

### Rust tests

Command:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Result:

Passed.

Summary:

```text
87 tests passed
0 failed
```

## Packaging

### Command

```powershell
npm run tauri build
```

### Result

Success.

### Artifacts

| Type | Path | Size |
|------|------|------|
| Standalone exe | `src-tauri/target/release/tauri-app.exe` | ~11.4 MB |
| MSI installer | `src-tauri/target/release/bundle/msi/tauri-app_0.1.0_x64_en-US.msi` | ~3.8 MB |
| NSIS installer | `src-tauri/target/release/bundle/nsis/tauri-app_0.1.0_x64-setup.exe` | ~2.5 MB |

### Signing Status

- Not signed.
- No signing certificate is configured in `tauri.conf.json` or environment variables.
- All three artifacts are unsigned test builds.

### Build Prerequisites Confirmed

- Node.js v24.14.0 + npm 11.9.0
- Rust stable toolchain via rustup
- Visual Studio Build Tools with C++ workload (provides MSVC and Windows SDK)
- WebView2 Runtime (pre-installed on Windows 10 21H2+ and Windows 11)
- Tauri CLI (`@tauri-apps/cli` v2)
- WiX Toolset v3: auto-downloaded by Tauri CLI on first build
- NSIS: auto-downloaded by Tauri CLI on first build

### Build Profile

- `release` profile with optimization
- `beforeBuildCommand`: `npm run build` (tsc + vite)
- Frontend dist served from `../dist`
- `frontendDist`: `../dist`

## PRD Compliance

- No API key saved, read, or uploaded.
- No system proxy modified.
- No `netsh winhttp set proxy` executed.
- No `npm config set registry` executed.
- No automatic PATH modification.
- No code changes; only documentation updated.

## PUA Self-Check

- Skipped validation: no
- Drifted outside scope: no
- Faked packaging result: no
- Faked signing status: no
- Committed PRD input file: no
- Committed `.claude/`: no
