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
