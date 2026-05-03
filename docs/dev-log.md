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
