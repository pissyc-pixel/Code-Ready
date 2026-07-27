# Code-Ready V2 Slice 1：首次向导与只读检测实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans`, `test-driven-development`, and `verification-before-completion` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. 执行模型固定为 GPT-5.6 Luna，推理强度 Max；每个行为变更必须先观察到本文指定的 RED，再写最小 GREEN。

**Goal:** 在 Windows 11 x64 与 macOS 13+ Apple Silicon 上提供相同语义的简体中文首次会话向导和只读状态中心，只检测 Git、Claude Code、Codex CLI，并通过版本化快照与可丢失/可重复事件保持 UI 可恢复。

**Architecture:** Rust 继续作为机器事实的唯一来源：平台边界枚举候选可执行文件，`ProcessRunner` 只运行固定的原生 `--version` 探针，工具检测器生成彼此独立的事实状态与版本状态，内存 `SnapshotStore` 原子发布版本化 `AppSnapshot`。事件只携带“快照已变化”的序号信号；React 对 sequence 去重、对缺口和重新聚焦重新调用 `bootstrap()`，并始终以较新的 snapshot 为真相。

**Tech Stack:** 现有 Tauri 2.11、Rust 2024/stable、serde、ts-rs 12.0.1、React 19.2.8、TypeScript 6.0.3、Vitest 4.1.10、Testing Library；不新增网络、持久化、shell、提权或遥测依赖。

## Global Constraints

- 基线必须是 `codex/v2-rebuild` 的 Slice 0 提交 `70ffb83`；执行前用 `git rev-parse HEAD` 核对。
- 支持平台语义固定为 Windows 11 x64 与 macOS 13.0+ Apple Silicon；两个平台使用同一组 DTO、事实状态、版本状态、向导步骤和中文文案。
- Slice 1 只检测 Git、Claude Code、Codex CLI；不得检测或操作 Winget、Node.js/npm。
- Claude Code 与 Codex CLI 只执行原生二进制的 `--version`；不得执行 `.cmd`、`.ps1`、shell script、Node、npm、包管理器或登录命令。
- 所有检测均在普通用户权限运行；不得请求 UAC、`sudo`、Authorization Services 或 Apple 安装弹窗。
- 不安装、不升级、不修复，不创建安装计划，不下载，不写配置，不读取旧配置，不实现代理、日志、helper、权限、更新或迁移。
- 工具事实状态严格限定为 `absent | presentHealthy | presentPathIssue | presentBroken | unknown`；检测运行状态不得写入工具事实。
- 版本状态严格限定为 `current | outdated | newerThanKnown | notComparable | unknown`；`presentHealthy` 不代表 `current`。
- Slice 1 没有签名版本基线，解析成功的版本必须是 `notComparable`；不得凭网络、当前日期或“看起来较新”推断 `current/outdated/newerThanKnown`。
- Rust serde DTO 是 IPC 唯一权威；TypeScript 只能由 `export-bindings` 单向生成，生成漂移必须使 CI 失败。
- 保持现有严格 CSP；Tauri capability 不增加 shell、process、fs、http、dialog、opener、updater 权限。
- 零遥测、零自动外联；原始 stdout/stderr 只在当前探针内有界保存并立即丢弃，不进入 DTO、事件或浏览器控制台。
- GitHub hosted runner 只能证明 Windows Server x64 与 macOS arm64 的 compile/test；不得冒充 Windows 11 或 macOS 13 干净机验收。

---

## 基线审计与冻结决策

### 当前树中确实存在的接口

- `apps/desktop/src-tauri/src/domain/contracts.rs` 当前导出 7 个 DTO；`BootstrapState` 只有 `schemaVersion/platform/tools`。
- `apps/desktop/src-tauri/src/domain/tool_registry.rs` 当前有 5 项产品注册表，并已锁定工具 ID 唯一、双平台 policy、能力不重复、依赖无环、Claude/Codex 无 Node 依赖、npm 不作为独立工具。
- `apps/desktop/src-tauri/src/platform/mod.rs` 当前 `PlatformAdapter` 只有 `platform()`；`FakePlatformAdapter` 只保存平台 ID。
- `apps/desktop/src-tauri/src/application/bootstrap.rs` 当前每次现场组装 `BootstrapState`，没有 snapshot store、检测运行或事件。
- `apps/desktop/src-tauri/src/api/commands.rs` 当前只有 `get_bootstrap_state`；`lib.rs` 只注册这一个 command。
- `apps/desktop/src/shared/api/client.ts` 当前只调用 `get_bootstrap_state`；`App.tsx` 只有 loading/error/基线摘要。
- `apps/desktop/src-tauri/capabilities/main.json` 当前权限精确为 `core:app:default`、`core:event:default`、`core:window:default`；事件能力已经存在，无需扩权。
- 根 `npm run check` 已包含 TS、lint、Vitest、Rust fmt/clippy/test 和 binding drift；CI 已在 `windows-2025` 与 `macos-15` 运行。
- Slice 0 计划曾写 TypeScript 7.0.2，但 `70ffb83` 当前 `apps/desktop/package.json` 实际锁定 6.0.3；Slice 1 以当前 manifest/lockfile 为准，不借检测功能升级工具链。
- 当前自动测试实际位于 Rust 模块内、`apps/desktop/src-tauri/tests/security_configuration.rs` 与 `apps/desktop/src/app/App.test.tsx`；树中没有现成 detector、process runner、snapshot 或 event-recovery test 可复用。

### Slice 1 已批准的两项边界

1. **首次会话而非持久化首次安装。** Slice 2 明确负责 SQLite 与“向导完成状态”。Slice 1 不使用 localStorage 绕过 Rust 事实来源；当前进程没有完成检测时展示欢迎页，首轮完成后展示状态中心。应用完全退出再启动会重新展示欢迎页。执行者不得把持久化提前到本切片。
2. **版本可见但不判断新旧。** 三个工具都解析并展示本机版本，但内置 `VersionPolicy::Unmanaged` 产生 `notComparable`。`current/outdated/newerThanKnown` 保留在契约和纯函数测试中，等待后续签名应用版本携带明确下限/已知上限；执行者不得自行引入版本基线。

### 官方命令与路径依据（核对日期：2026-07-27）

- Git 官方文档规定 `git --version` 打印 Git suite 版本：[git-scm.com/docs/git](https://git-scm.com/docs/git)。
- Apple 说明全新 macOS 直接调用 `/usr/bin/git` 可能触发 Command Line Tools 安装提示，因此本计划先用无副作用的 `/usr/bin/xcode-select --print-path` 验证 developer tools，再执行 Apple Git shim：[Installing the command-line tools](https://developer.apple.com/documentation/xcode/installing-the-command-line-tools/)。
- Anthropic 官方用 `claude --version` 验证安装；推荐原生位置为 macOS 的 `~/.local/bin/claude` 与 Windows 的 `%USERPROFILE%\.local\bin\claude.exe`：[Advanced setup](https://code.claude.com/docs/en/installation)、[Troubleshoot installation](https://code.claude.com/docs/en/troubleshoot-install)。
- OpenAI 官方独立安装器使用 `codex --version` 读取尾部版本。macOS 默认命令位置是 `~/.local/bin/codex`；Windows 默认位置是 `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin\codex.exe`：[Codex README](https://github.com/openai/codex/blob/main/README.md)、[install.sh](https://github.com/openai/codex/blob/main/scripts/install/install.sh)、[install.ps1](https://github.com/openai/codex/blob/main/scripts/install/install.ps1)。

### 固定事实与版本语义

| 观察 | 工具事实状态 | 版本 | 版本状态 |
|---|---|---|---|
| PATH 首选候选是原生二进制，`--version` 在 3 秒内 exit 0 | `presentHealthy` | 解析成功则保存，否则为空 | 成功解析为 `notComparable`，否则 `unknown` |
| PATH 无可运行原生候选，官方/内置已知位置的原生二进制 exit 0 | `presentPathIssue` | 同上 | 同上 |
| PATH 被脚本/`.cmd`/`.ps1` launcher 抢占，但已知原生位置健康 | `presentPathIssue` | 取健康原生二进制版本 | `notComparable` |
| 能确认存在原生二进制或非原生 launcher，但探针 nonzero、signal、timeout、permission denied 或只有非原生 launcher | `presentBroken` | 仅在输出可安全解析时保留；默认空 | `unknown` |
| PATH 和已知位置都没有候选；macOS 只有 Apple Git shim 且 `xcode-select --print-path` nonzero | `absent` | 空 | `unknown` |
| 平台枚举失败、路径不可读、二进制在枚举与 spawn 间消失、内部不变量失败 | `unknown` | 空 | `unknown` |

补充规则：

- exit 0 但版本文本无法解析时，工具仍是 `presentHealthy` 或 `presentPathIssue`；解析失败只影响版本状态。
- nonzero exit 永不解释成 `absent`；可执行文件已经存在，事实是 `presentBroken`。
- 3 秒超时先终止子进程并等待回收，再返回 `presentBroken`；检测不得无限挂起。
- `stdout` 与 `stderr` 每路最多读取 64 KiB，超出部分继续排空但不保留；解析顺序是 stdout 后 stderr。
- DTO 只暴露脱敏展示路径（用户主目录折叠为 `~` 或 `%USERPROFILE%`）、exit 类别和稳定 evidence code，不暴露原始输出。

## 文件结构映射

### Rust 新增

- `apps/desktop/src-tauri/src/domain/detection.rs`：事实状态、版本状态的纯领域规则和检测选择验证。
- `apps/desktop/src-tauri/src/domain/version.rs`：三个工具的版本解析与版本策略比较。
- `apps/desktop/src-tauri/src/application/snapshot_store.rs`：内存版本化快照、全局 sequence、原子 mutation。
- `apps/desktop/src-tauri/src/application/detection.rs`：检测协调器、单活动 run、后台执行、事件发布。
- `apps/desktop/src-tauri/src/platform/process.rs`：`ProcessRunner` 接口、native runner、有界输出与 timeout。
- `apps/desktop/src-tauri/src/platform/fake_process.rs`：按队列返回结果并记录请求的 `FakeProcessRunner`。
- `apps/desktop/src-tauri/src/tools/mod.rs`：只含 Git/Claude/Codex 的 typed detector registry。
- `apps/desktop/src-tauri/src/tools/shared.rs`：原生候选选择与固定 `--version` 探针。
- `apps/desktop/src-tauri/src/tools/git.rs`：Windows Git 与 macOS Apple shim 的窄差异。
- `apps/desktop/src-tauri/src/tools/claude.rs`：Claude 原生 CLI detector。
- `apps/desktop/src-tauri/src/tools/codex.rs`：Codex 原生 CLI detector。
- `apps/desktop/src-tauri/src/api/events.rs`：Tauri `detection.changed` event sink。
- `apps/desktop/src-tauri/tests/process_runner.rs`：native runner 的跨平台真实进程 contract test。

### Rust 修改

- `apps/desktop/src-tauri/src/domain/contracts.rs`：用 `AppSnapshot` 等 Slice 1 DTO 替代 `BootstrapState`。
- `apps/desktop/src-tauri/src/domain/tool_registry.rs`：保持 Slice 0 产品矩阵，同时增加 detector registry 覆盖不变量。
- `apps/desktop/src-tauri/src/domain/mod.rs`、`application/mod.rs`、`platform/mod.rs`、`api/mod.rs`、`lib.rs`：注册新模块与服务。
- `apps/desktop/src-tauri/src/platform/fake.rs`、`native.rs`：扩展候选枚举能力。
- `apps/desktop/src-tauri/src/application/bootstrap.rs`：改为从 store 返回 snapshot。
- `apps/desktop/src-tauri/src/api/commands.rs`：改为 `bootstrap` 与 `detect_tools` 两个窄 command。
- `apps/desktop/src-tauri/src/bin/export-bindings.rs`：导出新的权威 DTO 集合并清除旧 `BootstrapState.ts`。
- `apps/desktop/src-tauri/tests/security_configuration.rs`：锁定无新增能力、无网络/遥测依赖。

### React 新增

- `apps/desktop/src/features/detection/useDetectionSnapshot.ts`：bootstrap/event/focus 同步状态机。
- `apps/desktop/src/features/detection/useDetectionSnapshot.test.tsx`：重复、缺口、乱序、reload/focus、错误恢复测试。
- `apps/desktop/src/features/detection/presentation.ts`：事实状态和版本状态到 i18n key 的纯映射。
- `apps/desktop/src/features/onboarding/Onboarding.tsx`：欢迎、检测、解释三个阶段。
- `apps/desktop/src/features/onboarding/Onboarding.test.tsx`：首次会话向导与可访问性。
- `apps/desktop/src/features/dashboard/StatusCenter.tsx`：只读状态中心、单项/全量重检。
- `apps/desktop/src/features/dashboard/StatusCenter.test.tsx`：状态组合和双平台等价测试。
- `apps/desktop/src/shared/i18n/zh-CN.test.ts`：key 完整性和无协议状态漏译。

### React 修改

- `apps/desktop/src/shared/api/client.ts`：`bootstrap`、`detectTools`、`subscribeDetectionChanged`。
- `apps/desktop/src/shared/i18n/zh-CN.ts`：所有 Slice 1 简体中文资源与格式函数。
- `apps/desktop/src/app/App.tsx`、`App.test.tsx`、`styles.css`：组合新状态机和两个 feature。
- `apps/desktop/src/shared/api/generated/*.ts`：只允许 exporter 生成。

## 执行前基线门禁

在任何 RED 变更前运行：

```bash
git rev-parse HEAD
git status --short --branch
node --version
npm --version
rustc --version
npm ci
npm run check
```

Expected:

- HEAD 是 `70ffb83` 或已获批准、只包含本计划文档的直接后继。
- 工作树起始干净，且处于隔离 worktree；不得在 `main` 直接执行。
- Node 主版本至少 24，npm 主版本至少 11，Rust 至少 1.88。
- Slice 0 的 typecheck/lint/Vitest/fmt/clippy/Rust tests/binding drift 全部 PASS。

若 baseline check 失败，先保存完整命令和失败测试名并停止实现；不得把既有红灯与 Slice 1 的预期 RED 混在一起，也不得顺手升级依赖。

---

### Task 1：先固定 Slice 1 Rust DTO 与单向生成契约

**Files:**
- Modify: `apps/desktop/src-tauri/src/domain/contracts.rs:1-91`
- Modify: `apps/desktop/src-tauri/src/bin/export-bindings.rs:1-116`
- Generate/Delete: `apps/desktop/src/shared/api/generated/*.ts`
- Test: `apps/desktop/src-tauri/src/domain/contracts.rs`
- Test: `apps/desktop/src-tauri/src/bin/export-bindings.rs`

**Interfaces:**
- Consumes: 现有 `PlatformId`、`ToolId`、`ToolDefinition`。
- Produces: `AppSnapshot`、`ObservedToolState`、`VersionStatus`、`ToolObservation`、`DetectionEvidence`、`DetectionRun`、`DetectionRunStatus`、`DetectionEventEnvelope`、`CommandError`，供后续所有任务使用。

- [ ] **Step 1: 写 DTO 序列化 RED**

在 `contracts.rs` 先把旧 bootstrap 测试替换为下列精确断言；此时暂不定义新类型：

```rust
#[test]
fn slice_one_snapshot_keeps_fact_version_and_run_state_separate() {
    let snapshot = AppSnapshot {
        schema_version: 2,
        snapshot_version: 7,
        last_event_sequence: 11,
        platform: PlatformId::MacosArm64,
        tools: vec![],
        observations: vec![ToolObservation {
            tool_id: ToolId::ClaudeCode,
            state: ObservedToolState::PresentPathIssue,
            version: Some("2.1.89".into()),
            version_status: VersionStatus::NotComparable,
            evidence: DetectionEvidence {
                code: DetectionEvidenceCode::KnownLocationHealthy,
                display_path: Some("~/.local/bin/claude".into()),
                exit: Some(ProcessExitKind::Success),
            },
            checked_at_epoch_ms: 1_754_000_000_000,
        }],
        detection_run: Some(DetectionRun {
            id: "run-1".into(),
            requested_tool_ids: vec![ToolId::ClaudeCode],
            status: DetectionRunStatus::Running,
            started_at_epoch_ms: 1_754_000_000_000,
            finished_at_epoch_ms: None,
            error_code: None,
        }),
    };

    assert_eq!(
        serde_json::to_value(snapshot).unwrap(),
        serde_json::json!({
            "schemaVersion": 2,
            "snapshotVersion": 7,
            "lastEventSequence": 11,
            "platform": "macosArm64",
            "tools": [],
            "observations": [{
                "toolId": "claudeCode",
                "state": "presentPathIssue",
                "version": "2.1.89",
                "versionStatus": "notComparable",
                "evidence": {
                    "code": "knownLocationHealthy",
                    "displayPath": "~/.local/bin/claude",
                    "exit": "success"
                },
                "checkedAtEpochMs": 1_754_000_000_000_u64
            }],
            "detectionRun": {
                "id": "run-1",
                "requestedToolIds": ["claudeCode"],
                "status": "running",
                "startedAtEpochMs": 1_754_000_000_000_u64,
                "finishedAtEpochMs": null,
                "errorCode": null
            }
        })
    );
}

#[test]
fn detection_event_envelope_uses_the_stable_event_name() {
    let event = DetectionEventEnvelope {
        schema_version: 1,
        sequence: 12,
        snapshot_version: 8,
        emitted_at_epoch_ms: 1_754_000_000_100,
        event_type: DetectionEventType::DetectionChanged,
        run_id: "run-1".into(),
    };

    assert_eq!(
        serde_json::to_value(event).unwrap()["eventType"],
        "detection.changed"
    );
}
```

- [ ] **Step 2: 运行 RED 并确认失败原因**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml slice_one_snapshot_keeps_fact_version_and_run_state_separate
```

Expected: compile FAIL，首个错误是 `cannot find struct AppSnapshot`；若因拼写或旧测试失败，先修正测试直到只因新 DTO 缺失而红。

- [ ] **Step 3: 写最小 DTO GREEN**

在 `contracts.rs` 保留 Slice 0 的基础类型，删除 `BootstrapState`，增加以下精确类型；所有类型继续派生现有 serde/TS trait，并使用 camelCase：

```rust
pub enum ObservedToolState {
    Absent,
    PresentHealthy,
    PresentPathIssue,
    PresentBroken,
    Unknown,
}

pub enum VersionStatus {
    Current,
    Outdated,
    NewerThanKnown,
    NotComparable,
    Unknown,
}

pub enum DetectionEvidenceCode {
    PathCommandHealthy,
    KnownLocationHealthy,
    PathShadowed,
    NonNativeLauncher,
    CommandNonZero,
    CommandTimedOut,
    CommandLaunchFailed,
    VersionUnparseable,
    NotFound,
    PlatformProbeFailed,
    AppleDeveloperToolsMissing,
}

pub enum ProcessExitKind {
    Success,
    NonZero,
    Signalled,
    TimedOut,
    LaunchFailed,
}

pub struct DetectionEvidence {
    pub code: DetectionEvidenceCode,
    pub display_path: Option<String>,
    pub exit: Option<ProcessExitKind>,
}

pub struct ToolObservation {
    pub tool_id: ToolId,
    pub state: ObservedToolState,
    pub version: Option<String>,
    pub version_status: VersionStatus,
    pub evidence: DetectionEvidence,
    pub checked_at_epoch_ms: u64,
}

pub enum DetectionRunStatus {
    Running,
    Completed,
    Failed,
}

pub enum DetectionRunErrorCode {
    Internal,
}

pub struct DetectionRun {
    pub id: String,
    pub requested_tool_ids: Vec<ToolId>,
    pub status: DetectionRunStatus,
    pub started_at_epoch_ms: u64,
    pub finished_at_epoch_ms: Option<u64>,
    pub error_code: Option<DetectionRunErrorCode>,
}

pub struct AppSnapshot {
    pub schema_version: u16,
    pub snapshot_version: u64,
    pub last_event_sequence: u64,
    pub platform: PlatformId,
    pub tools: Vec<ToolDefinition>,
    pub observations: Vec<ToolObservation>,
    pub detection_run: Option<DetectionRun>,
}

pub enum DetectionEventType {
    #[serde(rename = "detection.changed")]
    #[ts(rename = "detection.changed")]
    DetectionChanged,
}

pub struct DetectionEventEnvelope {
    pub schema_version: u16,
    pub sequence: u64,
    pub snapshot_version: u64,
    pub emitted_at_epoch_ms: u64,
    pub event_type: DetectionEventType,
    pub run_id: String,
}

pub enum CommandErrorCode {
    InvalidToolSelection,
    DetectionAlreadyRunning,
    Internal,
}

pub struct CommandError {
    pub code: CommandErrorCode,
    pub retryable: bool,
}
```

对每个 enum/struct 添加与现有文件一致的 `Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS`、`#[serde(rename_all = "camelCase")]`、`#[ts(rename_all = "camelCase", export_to = "...ts")]`。`DetectionEventType` 的成员级 rename 优先于 rename_all。

- [ ] **Step 4: 更新 exporter 的精确类型集合**

将 exporter 的集合改为下列 19 项并逐项 `TS::export`；`clean_generated_types` 继续只删除 generated 目录内 `.ts`：

```rust
const CONTRACT_TYPE_NAMES: [&str; 19] = [
    "AppSnapshot",
    "CommandError",
    "CommandErrorCode",
    "DetectionEventEnvelope",
    "DetectionEventType",
    "DetectionEvidence",
    "DetectionEvidenceCode",
    "DetectionRun",
    "DetectionRunErrorCode",
    "DetectionRunStatus",
    "ObservedToolState",
    "PlatformId",
    "PlatformPolicy",
    "ProcessExitKind",
    "ToolCapability",
    "ToolDefinition",
    "ToolId",
    "ToolObservation",
    "VersionStatus",
];
```

Exporter 测试必须比较这一完整向量，并断言 `BootstrapState.ts` 不在生成目录。生成 `index.ts` 仍只含显式 `export type`。

- [ ] **Step 5: 生成 bindings 并验证旧文件被清除**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml slice_one_snapshot
npm run generate:bindings
npm run check:bindings
test ! -e apps/desktop/src/shared/api/generated/BootstrapState.ts
```

Expected: Rust tests PASS；生成目录恰有 20 个 `.ts` 文件（19 个类型加 `index.ts`）；`BootstrapState.ts` 不存在；drift check 退出码 0。

- [ ] **Step 6: 重构与回归**

Run:

```bash
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --all
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml contracts
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml export_bindings
```

Expected: PASS，且 `git diff --check` 无输出。

- [ ] **Step 7: 提交**

```bash
git add apps/desktop/src-tauri/src/domain/contracts.rs apps/desktop/src-tauri/src/bin/export-bindings.rs apps/desktop/src/shared/api/generated
git commit -m "feat: define detection snapshot contracts"
```

---

### Task 2：用纯测试固定版本解析和版本状态

**Files:**
- Create: `apps/desktop/src-tauri/src/domain/version.rs`
- Create: `apps/desktop/src-tauri/src/domain/detection.rs`
- Modify: `apps/desktop/src-tauri/src/domain/mod.rs`
- Test: `apps/desktop/src-tauri/src/domain/version.rs`
- Test: `apps/desktop/src-tauri/src/domain/detection.rs`

**Interfaces:**
- Consumes: `ToolId`、`VersionStatus`。
- Produces: `ParsedVersion::parse_for(tool_id, output)`、`VersionPolicy::classify(&ParsedVersion)`、`validate_detection_selection`。

- [ ] **Step 1: 写版本 fixture RED**

```rust
#[test]
fn parses_supported_native_cli_version_outputs() {
    assert_eq!(
        ParsedVersion::parse_for(ToolId::Git, b"git version 2.47.1.windows.1\n")
            .unwrap()
            .normalized(),
        "2.47.1.windows.1"
    );
    assert_eq!(
        ParsedVersion::parse_for(ToolId::ClaudeCode, b"2.1.89 (Claude Code)\n")
            .unwrap()
            .normalized(),
        "2.1.89"
    );
    assert_eq!(
        ParsedVersion::parse_for(ToolId::CodexCli, b"codex-cli 0.138.0\n")
            .unwrap()
            .normalized(),
        "0.138.0"
    );
}

#[test]
fn rejects_unrelated_or_ambiguous_output() {
    for (tool, bytes) in [
        (ToolId::Git, b"welcome 2.47.1".as_slice()),
        (ToolId::ClaudeCode, b"Claude Code".as_slice()),
        (ToolId::CodexCli, b"codex 0.1.0 extra".as_slice()),
    ] {
        assert!(ParsedVersion::parse_for(tool, bytes).is_none());
    }
}

#[test]
fn unmanaged_policy_never_claims_current() {
    let observed = ParsedVersion::parse_for(ToolId::CodexCli, b"codex-cli 0.138.0").unwrap();
    assert_eq!(
        VersionPolicy::Unmanaged.classify(&observed),
        VersionStatus::NotComparable
    );
}
```

- [ ] **Step 2: 运行 RED**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml parses_supported_native_cli_version_outputs
```

Expected: compile FAIL with unresolved `domain::version`/`ParsedVersion`。

- [ ] **Step 3: 写最小 parser GREEN**

实现不可 panic 的 ASCII parser：

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParsedVersion {
    numeric: Vec<u64>,
    normalized: String,
}

impl ParsedVersion {
    pub fn parse_for(tool_id: ToolId, output: &[u8]) -> Option<Self>;
    pub fn normalized(&self) -> &str;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionPolicy {
    Unmanaged,
    Range {
        minimum_supported: ParsedVersion,
        newest_known: ParsedVersion,
    },
}

impl VersionPolicy {
    pub fn classify(&self, observed: &ParsedVersion) -> VersionStatus;
}
```

固定解析规则：

- Git 必须以 ASCII `git version ` 开头，取其后第一行完整 token；token 必须以三段数字开头，允许 `.windows.1` 等 Git 平台后缀。
- Claude 必须是单行，接受 `X.Y.Z` 或 `X.Y.Z (Claude Code)`；不在任意句子中搜索数字。
- Codex 必须是单行 `codex-cli X.Y.Z` 或 `codex X.Y.Z`；版本 token 必须在行尾，兼容 `-alpha.N`/`-beta.N`。
- 数字段用 checked parse；溢出、NUL、多行附加文本、空输出、非 UTF-8一律返回 `None`。
- `Range`：低于 minimum 为 `Outdated`，高于 newest 为 `NewerThanKnown`，区间内为 `Current`；`Unmanaged` 始终为 `NotComparable`。

- [ ] **Step 4: 写检测选择 RED/GREEN**

先测试：

```rust
#[test]
fn slice_one_selection_is_non_empty_unique_and_limited_to_three_detectors() {
    assert_eq!(
        validate_detection_selection(None).unwrap(),
        vec![ToolId::Git, ToolId::ClaudeCode, ToolId::CodexCli]
    );
    assert!(validate_detection_selection(Some(vec![])).is_err());
    assert!(validate_detection_selection(Some(vec![ToolId::Git, ToolId::Git])).is_err());
    assert!(validate_detection_selection(Some(vec![ToolId::Nodejs])).is_err());
    assert!(validate_detection_selection(Some(vec![ToolId::Winget])).is_err());
}
```

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml slice_one_selection_is_non_empty_unique_and_limited_to_three_detectors
```

Expected: compile FAIL because function is absent。随后在 `domain/detection.rs` 实现：

```rust
pub const DETECTABLE_TOOL_IDS: [ToolId; 3] =
    [ToolId::Git, ToolId::ClaudeCode, ToolId::CodexCli];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetectionSelectionError {
    Empty,
    Duplicate(ToolId),
    Unsupported(ToolId),
}

pub fn validate_detection_selection(
    requested: Option<Vec<ToolId>>,
) -> Result<Vec<ToolId>, DetectionSelectionError>;
```

`None` 返回常量顺序；`Some` 保留用户顺序但拒绝空、重复和非本切片工具。

- [ ] **Step 5: 验证与提交**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
git add apps/desktop/src-tauri/src/domain
git commit -m "feat: classify detected tool versions"
```

Expected: PASS，无 warning。

---

### Task 3：扩展 PlatformAdapter，只在边界枚举原生候选

**Files:**
- Modify: `apps/desktop/src-tauri/src/platform/mod.rs:1-8`
- Modify: `apps/desktop/src-tauri/src/platform/fake.rs:1-20`
- Modify: `apps/desktop/src-tauri/src/platform/native.rs:1-55`
- Test: `apps/desktop/src-tauri/src/platform/fake.rs`
- Test: `apps/desktop/src-tauri/src/platform/native.rs`

**Interfaces:**
- Consumes: `PlatformId`、`ToolId`。
- Produces: `PlatformAdapter::tool_candidates`、`ExecutableCandidate`、`CandidateOrigin`、`CandidateKind`；后续 detector 不自行读 PATH、HOME、USERPROFILE、LOCALAPPDATA 或文件 magic。

- [ ] **Step 1: 写双平台候选 RED**

```rust
#[test]
fn fake_adapter_returns_configured_candidates_without_host_cfg() {
    let claude = ExecutableCandidate::native(
        "/fake/home/.local/bin/claude",
        "~/.local/bin/claude",
        CandidateOrigin::KnownLocation,
    );
    let adapter = FakePlatformAdapter::builder(PlatformId::WindowsX64)
        .with_candidates(ToolId::ClaudeCode, vec![claude.clone()])
        .build();

    assert_eq!(
        adapter.tool_candidates(ToolId::ClaudeCode).unwrap(),
        vec![claude]
    );
    assert_eq!(adapter.platform(), PlatformId::WindowsX64);
}

#[test]
fn native_known_locations_match_official_cli_installers() {
    assert_eq!(
        known_location_templates(PlatformId::MacosArm64, ToolId::ClaudeCode),
        vec!["$HOME/.local/bin/claude"]
    );
    assert_eq!(
        known_location_templates(PlatformId::WindowsX64, ToolId::ClaudeCode),
        vec!["%USERPROFILE%\\.local\\bin\\claude.exe"]
    );
    assert_eq!(
        known_location_templates(PlatformId::MacosArm64, ToolId::CodexCli),
        vec!["$HOME/.local/bin/codex"]
    );
    assert_eq!(
        known_location_templates(PlatformId::WindowsX64, ToolId::CodexCli),
        vec!["%LOCALAPPDATA%\\Programs\\OpenAI\\Codex\\bin\\codex.exe"]
    );
}
```

- [ ] **Step 2: 运行 RED**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml fake_adapter_returns_configured_candidates_without_host_cfg
```

Expected: compile FAIL because builder/candidate types do not exist。

- [ ] **Step 3: 实现精确 adapter 接口**

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateOrigin {
    Path,
    KnownLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateKind {
    NativeBinary,
    NonNativeLauncher,
    AppleGitShim,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableCandidate {
    pub path: PathBuf,
    pub display_path: String,
    pub origin: CandidateOrigin,
    pub kind: CandidateKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformProbeError {
    EnvironmentUnavailable,
    PathUnreadable,
}

pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> PlatformId;
    fn tool_candidates(
        &self,
        tool_id: ToolId,
    ) -> Result<Vec<ExecutableCandidate>, PlatformProbeError>;
}
```

`FakePlatformAdapter::builder` 保存每个 ToolId 的结果，可配置成功候选或 `PlatformProbeError`。不得向 fake 加只供断言内部状态的 production 方法。

- [ ] **Step 4: 实现 native 候选规则**

实现纯 helper 并用 fixture byte 测试：

- PATH 顺序必须保持；同一路径去重；已知位置只在 PATH 候选之后追加。
- Windows 按 PATHEXT/常见 shell 入口识别 `.exe`、`.cmd`、`.bat`、`.ps1`，但只有 PE `MZ` 的 `.exe` 标为 `NativeBinary`；其他已有入口标为 `NonNativeLauncher`，后续绝不执行。
- macOS 解析 PATH 中同名文件并跟随 symlink；Mach-O 64/fat magic 标为 `NativeBinary`，shebang/文本 launcher 标为 `NonNativeLauncher`。
- `/usr/bin/git` 在 macOS 标为 `AppleGitShim`，不能直接当健康二进制执行。
- Git 已知位置：
  - Windows：`%ProgramFiles%\Git\cmd\git.exe`、`%ProgramFiles%\Git\bin\git.exe`。
  - macOS：`/opt/homebrew/bin/git`、`/usr/local/bin/git`、`/usr/bin/git`。
- 用户目录展示路径折叠为 `~` 或 `%USERPROFILE%`；其他路径保留本地可读形式。
- 环境变量缺失不能 panic。某一可选已知根缺失只跳过该候选；读取 PATH 本身失败才返回 `EnvironmentUnavailable`。

不使用 `where.exe`、`which`、PowerShell、`sh` 或 Tauri plugin。

- [ ] **Step 5: 增加边界架构守卫**

在现有 `domain_and_application_sources_do_not_cross_platform_or_tauri_boundaries` 基础上增加断言：

```rust
for module in ["domain", "application"] {
    assert_source_tree_has_no_forbidden_tokens(
        &source_root.join(module),
        &[
            "std::env".into(),
            "std::fs".into(),
            "target_os".into(),
            "USERPROFILE".into(),
            "LOCALAPPDATA".into(),
        ],
    );
}
```

保留原有禁止项；如果 `snapshot_store` 需要标准同步类型，不得因此放宽平台/进程禁令。

- [ ] **Step 6: 验证与提交**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml platform
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain_and_application_sources_do_not_cross_platform_or_tauri_boundaries
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
git add apps/desktop/src-tauri/src/platform apps/desktop/src-tauri/src/application/bootstrap.rs
git commit -m "feat: enumerate native tool candidates"
```

Expected: PASS；平台环境和文件格式判断只出现在 `platform/`。

---

### Task 4：用 FakeProcessRunner 驱动固定命令、timeout 和有界输出

**Files:**
- Create: `apps/desktop/src-tauri/src/platform/process.rs`
- Create: `apps/desktop/src-tauri/src/platform/fake_process.rs`
- Create: `apps/desktop/src-tauri/tests/process_runner.rs`
- Modify: `apps/desktop/src-tauri/src/platform/mod.rs`
- Test: same files

**Interfaces:**
- Consumes: 绝对可执行路径。
- Produces: `ProcessRunner::run(ProcessRequest) -> ProcessOutcome`；任何工具 detector 只能通过此接口运行进程。

- [ ] **Step 1: 写 FakeProcessRunner RED**

```rust
#[test]
fn fake_process_runner_returns_queued_results_and_records_exact_request() {
    let runner = FakeProcessRunner::new([ProcessOutcome::Exited {
        code: Some(0),
        stdout: b"codex-cli 0.138.0\n".to_vec(),
        stderr: vec![],
        stdout_truncated: false,
        stderr_truncated: false,
    }]);
    let request = ProcessRequest::version_probe("/fake/codex");

    assert!(matches!(runner.run(request.clone()), ProcessOutcome::Exited { code: Some(0), .. }));
    assert_eq!(runner.requests(), vec![request]);
}

#[test]
fn version_probe_has_no_shell_or_mutating_arguments() {
    let request = ProcessRequest::version_probe("/fake/claude");
    assert_eq!(request.args, vec!["--version"]);
    assert_eq!(request.timeout, Duration::from_secs(3));
    assert!(request.environment.is_empty());
}
```

- [ ] **Step 2: 运行 RED**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml fake_process_runner_returns_queued_results_and_records_exact_request
```

Expected: compile FAIL because `ProcessRunner` types are absent。

- [ ] **Step 3: 实现接口与 fake GREEN**

```rust
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
pub const OUTPUT_LIMIT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessRequest {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub timeout: Duration,
    pub environment: Vec<(OsString, OsString)>,
}

impl ProcessRequest {
    pub fn version_probe(path: impl Into<PathBuf>) -> Self;
    pub fn apple_developer_dir_probe() -> Self;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchFailureKind {
    NotFound,
    PermissionDenied,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessOutcome {
    Exited {
        code: Option<i32>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        stdout_truncated: bool,
        stderr_truncated: bool,
    },
    TimedOut,
    LaunchFailed(LaunchFailureKind),
}

pub trait ProcessRunner: Send + Sync {
    fn run(&self, request: ProcessRequest) -> ProcessOutcome;
}
```

`apple_developer_dir_probe()` 必须精确为 `/usr/bin/xcode-select --print-path`、3 秒、空环境覆盖。`FakeProcessRunner` 用 `Mutex<VecDeque<ProcessOutcome>>` 和 `Mutex<Vec<ProcessRequest>>`；队列耗尽返回 `LaunchFailed(Other)`，不得 panic。

- [ ] **Step 4: 写 native runner RED**

在 integration test 中让测试二进制自举为 fixture：

```rust
#[test]
fn probe_fixture_entrypoint() {
    match std::env::var("CODE_READY_PROCESS_FIXTURE").as_deref() {
        Ok("success") => print!("{}", "x".repeat(70 * 1024)),
        Ok("nonzero") => std::process::exit(23),
        Ok("timeout") => std::thread::sleep(Duration::from_secs(1)),
        _ => {}
    }
}

#[test]
fn native_runner_caps_output_and_reports_timeout_and_nonzero() {
    let executable = std::env::current_exe().unwrap();
    let runner = NativeProcessRunner;

    let success = runner.run(fixture_request(&executable, "success", Duration::from_secs(3)));
    assert!(matches!(
        success,
        ProcessOutcome::Exited {
            code: Some(0),
            stdout_truncated: true,
            ..
        }
    ));

    assert!(matches!(
        runner.run(fixture_request(&executable, "nonzero", Duration::from_secs(3))),
        ProcessOutcome::Exited { code: Some(23), .. }
    ));
    assert_eq!(
        runner.run(fixture_request(&executable, "timeout", Duration::from_millis(30))),
        ProcessOutcome::TimedOut
    );
}
```

`fixture_request` 参数是当前测试二进制、`--exact probe_fixture_entrypoint --nocapture` 和单个 fixture 环境变量。

- [ ] **Step 5: 实现 native runner GREEN**

只用标准库：

1. `Command` 直接执行 `request.executable`，逐项传 `args`，stdin 为 null，stdout/stderr piped；不经过 shell。
2. spawn 后分别启动两个 reader thread，持续排空 pipe，只保留前 64 KiB并记录 truncated。
3. 主线程用 `Instant` + `child.try_wait()` 轮询，间隔最多 10 ms。
4. deadline 到达先 `child.kill()`，再 `child.wait()` 回收，然后 join 两个 reader，返回 `TimedOut`。
5. spawn 的 `NotFound`、`PermissionDenied`、其他 `io::ErrorKind` 映射到稳定 `LaunchFailureKind`。
6. 正常结束时返回真实 `ExitStatus::code()`；Unix signal 为 `code: None`。
7. 不保存完整 command line，不打印 stdout/stderr，不调用日志/遥测。

- [ ] **Step 6: 验证与提交**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml fake_process
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test process_runner
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
git add apps/desktop/src-tauri/src/platform apps/desktop/src-tauri/tests/process_runner.rs
git commit -m "feat: run bounded read-only probes"
```

Expected: 两个平台可编译的测试 PASS；timeout 测试在 500 ms 内返回；无 zombie/handle 泄漏 warning。

---

### Task 5：用跨平台矩阵驱动 Git、Claude、Codex detector

**Files:**
- Create: `apps/desktop/src-tauri/src/tools/mod.rs`
- Create: `apps/desktop/src-tauri/src/tools/shared.rs`
- Create: `apps/desktop/src-tauri/src/tools/git.rs`
- Create: `apps/desktop/src-tauri/src/tools/claude.rs`
- Create: `apps/desktop/src-tauri/src/tools/codex.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/domain/tool_registry.rs`
- Test: all files above

**Interfaces:**
- Consumes: `PlatformAdapter`、`ProcessRunner`、`ParsedVersion`、`ToolObservation`。
- Produces: `ToolDetector::detect` 和恰含 Git/Claude/Codex 的 `built_in_detector_registry()`。

- [ ] **Step 1: 写共享事实矩阵 RED**

在 `tools/shared.rs` 的 test module 用 FakePlatformAdapter + FakeProcessRunner 写表驱动测试：

```rust
struct Case {
    name: &'static str,
    candidates: Result<Vec<ExecutableCandidate>, PlatformProbeError>,
    outcomes: Vec<ProcessOutcome>,
    expected_state: ObservedToolState,
    expected_version: Option<&'static str>,
    expected_version_status: VersionStatus,
    expected_evidence: DetectionEvidenceCode,
}

#[test]
fn native_cli_fact_matrix_is_platform_independent() {
    let cases = [
        Case {
            name: "path native success",
            candidates: Ok(vec![native_path("/path/claude")]),
            outcomes: vec![success("2.1.89 (Claude Code)\n")],
            expected_state: ObservedToolState::PresentHealthy,
            expected_version: Some("2.1.89"),
            expected_version_status: VersionStatus::NotComparable,
            expected_evidence: DetectionEvidenceCode::PathCommandHealthy,
        },
        Case {
            name: "known native success",
            candidates: Ok(vec![native_known("/home/.local/bin/claude")]),
            outcomes: vec![success("2.1.89\n")],
            expected_state: ObservedToolState::PresentPathIssue,
            expected_version: Some("2.1.89"),
            expected_version_status: VersionStatus::NotComparable,
            expected_evidence: DetectionEvidenceCode::KnownLocationHealthy,
        },
        Case {
            name: "not found",
            candidates: Ok(vec![]),
            outcomes: vec![],
            expected_state: ObservedToolState::Absent,
            expected_version: None,
            expected_version_status: VersionStatus::Unknown,
            expected_evidence: DetectionEvidenceCode::NotFound,
        },
        Case {
            name: "nonzero",
            candidates: Ok(vec![native_path("/path/claude")]),
            outcomes: vec![nonzero(2)],
            expected_state: ObservedToolState::PresentBroken,
            expected_version: None,
            expected_version_status: VersionStatus::Unknown,
            expected_evidence: DetectionEvidenceCode::CommandNonZero,
        },
        Case {
            name: "timeout",
            candidates: Ok(vec![native_path("/path/claude")]),
            outcomes: vec![ProcessOutcome::TimedOut],
            expected_state: ObservedToolState::PresentBroken,
            expected_version: None,
            expected_version_status: VersionStatus::Unknown,
            expected_evidence: DetectionEvidenceCode::CommandTimedOut,
        },
        Case {
            name: "platform failure",
            candidates: Err(PlatformProbeError::PathUnreadable),
            outcomes: vec![],
            expected_state: ObservedToolState::Unknown,
            expected_version: None,
            expected_version_status: VersionStatus::Unknown,
            expected_evidence: DetectionEvidenceCode::PlatformProbeFailed,
        },
    ];

    for platform in [PlatformId::WindowsX64, PlatformId::MacosArm64] {
        for case in &cases {
            let observation = run_case(platform.clone(), case);
            assert_eq!(observation.state, case.expected_state, "{}", case.name);
            assert_eq!(observation.version.as_deref(), case.expected_version, "{}", case.name);
            assert_eq!(observation.version_status, case.expected_version_status, "{}", case.name);
            assert_eq!(observation.evidence.code, case.expected_evidence, "{}", case.name);
        }
    }
}
```

另写两个独立测试：

```rust
#[test]
fn exit_zero_with_unparseable_version_keeps_tool_health_separate() {
    let observation = detect_with(success("Claude Code\n"));
    assert_eq!(observation.state, ObservedToolState::PresentHealthy);
    assert_eq!(observation.version, None);
    assert_eq!(observation.version_status, VersionStatus::Unknown);
    assert_eq!(observation.evidence.code, DetectionEvidenceCode::VersionUnparseable);
}

#[test]
fn non_native_launcher_is_never_executed() {
    let runner = FakeProcessRunner::new([]);
    let observation = detect_candidates(
        vec![non_native_path("/path/claude"), native_known("/home/.local/bin/claude")],
        &runner,
        [success("2.1.89\n")],
    );
    assert_eq!(runner.requests().len(), 1);
    assert_eq!(runner.requests()[0].executable, PathBuf::from("/home/.local/bin/claude"));
    assert_eq!(observation.state, ObservedToolState::PresentPathIssue);
    assert_eq!(observation.evidence.code, DetectionEvidenceCode::PathShadowed);
}
```

- [ ] **Step 2: 运行 RED**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml native_cli_fact_matrix_is_platform_independent
```

Expected: compile FAIL because `tools` module and detector helper are absent。

- [ ] **Step 3: 实现 detector 接口和共享 GREEN**

```rust
pub trait Clock: Send + Sync {
    fn now_epoch_ms(&self) -> u64;
}

pub struct DetectionContext<'a> {
    pub platform: &'a dyn PlatformAdapter,
    pub runner: &'a dyn ProcessRunner,
    pub clock: &'a dyn Clock,
}

pub trait ToolDetector: Send + Sync {
    fn tool_id(&self) -> ToolId;
    fn detect(&self, context: &DetectionContext<'_>) -> ToolObservation;
}
```

`shared::probe_native_version` 必须：

1. 从 adapter 获取候选，不读环境。
2. 记录 PATH 中首个 launcher 是否非原生。
3. 只对 `NativeBinary` 运行绝对路径加 `--version`。
4. PATH 原生成功为 healthy；known 原生成功为 path issue；被非原生 PATH shadow 时使用 `PathShadowed`。
5. 所有候选都不存在为 absent。
6. 只有非原生 launcher 为 broken/`NonNativeLauncher`，runner 请求数保持 0。
7. 按本计划事实表映射 nonzero/signal/timeout/launch failure。
8. 成功 exit 与版本 parser 结果分开映射。
9. 使用 `VersionPolicy::Unmanaged`；不访问网络。
10. observation 时间只来自 `Clock`。

在 `platform/process.rs` 同时提供 `SystemClock`；测试使用固定返回 `1_754_000_000_000` 的 `FakeClock`。

- [ ] **Step 4: 写三个工具的命令 RED**

```rust
#[test]
fn each_slice_one_detector_runs_only_its_native_version_command() {
    for (detector, executable) in [
        (boxed_git_detector(), "/fake/git"),
        (boxed_claude_detector(), "/fake/claude"),
        (boxed_codex_detector(), "/fake/codex"),
    ] {
        let (observation, requests) = detect_one(detector, executable);
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].executable, PathBuf::from(executable));
        assert_eq!(requests[0].args, vec![OsString::from("--version")]);
        assert_eq!(observation.state, ObservedToolState::PresentHealthy);
    }
}
```

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml each_slice_one_detector_runs_only_its_native_version_command
```

Expected: FAIL because concrete detectors are absent。

- [ ] **Step 5: 实现 Git/Claude/Codex detector**

- `ClaudeDetector` 与 `CodexDetector` 只调用共享原生探针，各自传入工具 ID 和 parser。
- Windows Git 走共享原生探针。
- macOS Git：
  1. 若存在非 Apple shim 的 PATH/known native Git，按共享规则检测。
  2. 若候选只剩 `AppleGitShim`，先运行固定 `/usr/bin/xcode-select --print-path`。
  3. preflight exit 0 且输出是单个绝对 developer directory 才运行 `/usr/bin/git --version`。
  4. preflight nonzero 映射 `absent/AppleDeveloperToolsMissing`，且不得运行 git，避免 Apple 安装弹窗。
  5. preflight timeout/launch failure/不可解析路径映射 `unknown/PlatformProbeFailed`。
  6. Git `--version` nonzero/timeout 仍映射 broken。

不得调用 `xcode-select --install`、`pkgutil`、`xcrun` 或任何包管理器。

- [ ] **Step 6: 用 typed detector registry 锁定范围**

`built_in_detector_registry()` 返回稳定顺序 `[Git, ClaudeCode, CodexCli]`。测试必须断言：

```rust
#[test]
fn detector_registry_exactly_matches_slice_one_scope() {
    let ids = built_in_detector_registry()
        .iter()
        .map(|detector| detector.tool_id())
        .collect::<Vec<_>>();
    assert_eq!(ids, DETECTABLE_TOOL_IDS);
    assert_eq!(ids.len(), 3);
    assert!(!ids.contains(&ToolId::Winget));
    assert!(!ids.contains(&ToolId::Nodejs));
}
```

再把该 registry 与现有 5 项产品 registry 交叉检查：detector 的每个 ID 必须存在、拥有 `Detect` capability、在两个平台都不是 `Unavailable`。不得删除 Slice 0 的五项未来产品定义或放宽原有不变量。

- [ ] **Step 7: 加 Node/npm 禁令守卫**

```bash
rg -n -i "node(js)?|npm|\\.cmd|\\.ps1|powershell|/bin/sh|cmd\\.exe" \
  apps/desktop/src-tauri/src/tools/claude.rs \
  apps/desktop/src-tauri/src/tools/codex.rs
```

Expected: 无输出。把同一规则做成 Rust source guard test（通过拼接 token 避免测试源码自身命中）。Git detector 允许字符串 `AppleGitShim`，不允许 shell。

- [ ] **Step 8: 验证与提交**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml tools
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml tool_registry
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
git add apps/desktop/src-tauri/src/tools apps/desktop/src-tauri/src/domain/tool_registry.rs apps/desktop/src-tauri/src/lib.rs
git commit -m "feat: detect native developer tools"
```

Expected: 所有 fixture PASS，三个 detector 不执行 Node/npm 或 shell。

---

### Task 6：用版本化 SnapshotStore 固定 sequence 和事实更新

**Files:**
- Create: `apps/desktop/src-tauri/src/application/snapshot_store.rs`
- Modify: `apps/desktop/src-tauri/src/application/mod.rs`
- Modify: `apps/desktop/src-tauri/src/application/bootstrap.rs:1-109`
- Test: `apps/desktop/src-tauri/src/application/snapshot_store.rs`
- Test: `apps/desktop/src-tauri/src/application/bootstrap.rs`

**Interfaces:**
- Consumes: `AppSnapshot`、`ToolObservation`、`DetectionRun`。
- Produces: `SnapshotStore::snapshot`、`begin_run`、`record_observation`、`finish_run`；每次 mutation 返回对应 `DetectionEventEnvelope`。

- [ ] **Step 1: 写 store RED**

```rust
#[test]
fn snapshot_store_versions_each_mutation_and_never_changes_on_read() {
    let clock = Arc::new(FakeClock::new([100, 110, 120]));
    let store = SnapshotStore::new(
        PlatformId::WindowsX64,
        built_in_tool_registry(),
        clock,
    );

    let initial = store.snapshot();
    assert_eq!(initial.schema_version, 2);
    assert_eq!(initial.snapshot_version, 1);
    assert_eq!(initial.last_event_sequence, 0);
    assert!(initial.observations.is_empty());
    assert!(initial.detection_run.is_none());
    assert_eq!(store.snapshot(), initial);

    let started = store.begin_run("run-1", vec![ToolId::Git]).unwrap();
    assert_eq!(started.sequence, 1);
    assert_eq!(started.snapshot_version, 2);
    assert_eq!(store.snapshot().last_event_sequence, 1);
}

#[test]
fn recording_an_observation_replaces_only_the_same_tool_fact() {
    let store = store_with_running_run("run-1", [ToolId::Git, ToolId::CodexCli]);
    store.record_observation("run-1", healthy_git("2.47.1")).unwrap();
    store.record_observation("run-1", broken_codex()).unwrap();

    let snapshot = store.snapshot();
    assert_eq!(snapshot.observations.len(), 2);
    assert_eq!(snapshot.observations[0].tool_id, ToolId::Git);
    assert_eq!(snapshot.observations[1].tool_id, ToolId::CodexCli);
    assert_eq!(snapshot.detection_run.unwrap().status, DetectionRunStatus::Running);
}
```

- [ ] **Step 2: 运行 RED**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml snapshot_store_versions_each_mutation_and_never_changes_on_read
```

Expected: compile FAIL because `SnapshotStore` is absent。

- [ ] **Step 3: 实现原子 store GREEN**

```rust
pub struct SnapshotStore {
    inner: Mutex<StoreState>,
    clock: Arc<dyn Clock>,
}

struct StoreState {
    snapshot: AppSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotMutationError {
    RunAlreadyActive,
    RunNotActive,
    RunIdMismatch,
    ToolNotRequested,
}

impl SnapshotStore {
    pub fn new(
        platform: PlatformId,
        tools: Vec<ToolDefinition>,
        clock: Arc<dyn Clock>,
    ) -> Self;
    pub fn snapshot(&self) -> AppSnapshot;
    pub fn begin_run(
        &self,
        run_id: &str,
        tool_ids: Vec<ToolId>,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError>;
    pub fn record_observation(
        &self,
        run_id: &str,
        observation: ToolObservation,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError>;
    pub fn finish_run(
        &self,
        run_id: &str,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError>;
    pub fn fail_run(
        &self,
        run_id: &str,
        error_code: DetectionRunErrorCode,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError>;
}
```

固定 mutation 规则：

- 初始 snapshotVersion=1、lastEventSequence=0、observations 空、run 空。
- 每次成功 mutation 在同一 mutex 临界区将 snapshotVersion 和 lastEventSequence 各加 1，event 使用更新后的值。
- checked arithmetic 溢出返回内部错误，不回绕。
- `begin_run` 只在没有 Running run 时成功。
- `record_observation` 必须匹配 active run ID 和 requested tool；按 `DETECTABLE_TOOL_IDS` 稳定顺序 upsert。
- 新 run 不清空旧事实；运行状态说明正在复检，旧 observation 仍是最近已知事实。
- `finish_run` 只把任务状态改为 Completed、清空 run error 并写 finished timestamp；不改任何工具事实。
- `fail_run` 只把任务状态改为 Failed、写 `Internal` 和 finished timestamp；不改任何工具事实。
- event 的 emitted timestamp 来自 mutation 时的 clock，event 只含 metadata。

- [ ] **Step 4: bootstrap 改为无副作用 snapshot read**

先修改测试：

```rust
#[test]
fn bootstrap_returns_the_current_snapshot_without_advancing_versions() {
    let store = Arc::new(snapshot_store(PlatformId::MacosArm64));
    let service = BootstrapService::new(store.clone());
    let before = store.snapshot();

    assert_eq!(service.get_snapshot(), before);
    assert_eq!(service.get_snapshot(), before);
}
```

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap_returns_the_current_snapshot_without_advancing_versions
```

Expected: FAIL，因为 service 仍组装旧 `BootstrapState`。最小实现：

```rust
pub struct BootstrapService {
    store: Arc<SnapshotStore>,
}

impl BootstrapService {
    pub fn new(store: Arc<SnapshotStore>) -> Self;
    pub fn get_snapshot(&self) -> AppSnapshot {
        self.store.snapshot()
    }
}
```

- [ ] **Step 5: 重构与并发验证**

增加 8 个线程并发读取、单线程连续 mutation 的测试；断言读到的每个 snapshot 都满足 `snapshotVersion == lastEventSequence + 1`，且 observation 不出现半写状态。

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml snapshot_store
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap
```

Expected: PASS。

- [ ] **Step 6: 提交**

```bash
git add apps/desktop/src-tauri/src/application
git commit -m "feat: store versioned detection snapshots"
```

---

### Task 7：检测协调器保证单活动 run，事件失败不影响快照

**Files:**
- Create: `apps/desktop/src-tauri/src/application/detection.rs`
- Modify: `apps/desktop/src-tauri/src/application/mod.rs`
- Test: `apps/desktop/src-tauri/src/application/detection.rs`

**Interfaces:**
- Consumes: detector registry、`SnapshotStore`、`ProcessRunner`、`PlatformAdapter`、`Clock`。
- Produces: `DetectionService::start(requested) -> Result<String, StartDetectionError>`；`DetectionEventSink`。

- [ ] **Step 1: 写 orchestration RED**

```rust
#[test]
fn full_detection_returns_immediately_then_publishes_snapshot_events_in_order() {
    let harness = DetectionHarness::new(PlatformId::MacosArm64)
        .with_healthy_versions([
            (ToolId::Git, "git version 2.47.1"),
            (ToolId::ClaudeCode, "2.1.89 (Claude Code)"),
            (ToolId::CodexCli, "codex-cli 0.138.0"),
        ]);

    let run_id = harness.service.start(None).unwrap();
    harness.wait_until_completed(&run_id);

    let snapshot = harness.store.snapshot();
    assert_eq!(
        snapshot
            .observations
            .iter()
            .map(|item| item.tool_id.clone())
            .collect::<Vec<_>>(),
        vec![ToolId::Git, ToolId::ClaudeCode, ToolId::CodexCli]
    );
    assert_eq!(snapshot.detection_run.unwrap().status, DetectionRunStatus::Completed);
    assert_eq!(
        harness.events.sequences(),
        vec![1, 2, 3, 4, 5]
    );
}

#[test]
fn dropped_or_failed_event_delivery_never_rolls_back_snapshot_truth() {
    let harness = DetectionHarness::with_sink(AlwaysFailingEventSink);
    let run_id = harness.service.start(Some(vec![ToolId::Git])).unwrap();
    harness.wait_until_completed(&run_id);

    let snapshot = harness.store.snapshot();
    assert_eq!(snapshot.observations.len(), 1);
    assert_eq!(snapshot.last_event_sequence, 3);
    assert_eq!(snapshot.snapshot_version, 4);
}

#[test]
fn a_second_run_is_rejected_without_changing_tool_facts() {
    let harness = DetectionHarness::blocked_runner();
    let first = harness.service.start(Some(vec![ToolId::Git])).unwrap();
    assert_eq!(
        harness.service.start(Some(vec![ToolId::CodexCli])),
        Err(StartDetectionError::AlreadyRunning(first))
    );
    assert!(harness.store.snapshot().observations.is_empty());
}
```

Sequence 数量解释：begin=1，三个 observation=2/3/4，finish=5；单工具则 begin/observation/finish 共 3。

- [ ] **Step 2: 运行 RED**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml full_detection_returns_immediately_then_publishes_snapshot_events_in_order
```

Expected: compile FAIL because `DetectionService` is absent。

- [ ] **Step 3: 实现最小协调器 GREEN**

```rust
pub trait DetectionEventSink: Send + Sync {
    fn publish(&self, event: DetectionEventEnvelope) -> Result<(), EventPublishError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartDetectionError {
    InvalidSelection(DetectionSelectionError),
    AlreadyRunning(String),
    Internal,
}

pub struct DetectionService {
    store: Arc<SnapshotStore>,
    platform: Arc<dyn PlatformAdapter>,
    runner: Arc<dyn ProcessRunner>,
    clock: Arc<dyn Clock>,
    detectors: Arc<Vec<Arc<dyn ToolDetector>>>,
    sink: Arc<dyn DetectionEventSink>,
    next_run_id: AtomicU64,
}

impl DetectionService {
    pub fn start(&self, requested: Option<Vec<ToolId>>) -> Result<String, StartDetectionError>;
}
```

实现顺序：

1. 同步验证 selection。
2. 用 `run-{counter}` 生成当前进程内唯一 ID；counter checked increment。
3. 同步 `begin_run`，再 best-effort publish；publish error 被丢弃，不回滚 store。
4. clone 所需 Arc，启动命名 `code-ready-detect-{id}` 的 `std::thread`，command 立即返回 ID。
5. worker 按 selection 顺序串行调用 detector；每个 observation 落 store 后 best-effort publish。
6. 最后 `finish_run` 并 publish。
7. detector 内部可预期失败必须映射 observation，不得使 worker 提前退出。
8. thread spawn 失败时立即 `fail_run(Internal)` 并返回 `StartDetectionError::Internal`；已产生的 begin/fail event 都是 best-effort。
9. worker 中 store 不变量错误必须调用 `fail_run(Internal)` 后停止；不得让 run 永久停在 Running。
10. 由于 Slice 1 没日志，不打印原始错误或输出。

`AlreadyRunning` 由 store 当前 run 得出，不维护第二份 boolean。

增加测试：模拟 thread spawner 返回错误，断言 command 返回 internal、snapshot run 为 Failed/errorCode Internal、observations 不变。为此只注入窄 `ThreadSpawner` trait；production 使用 `std::thread::Builder`，fake 只返回成功或失败，不执行检测逻辑。

- [ ] **Step 4: 增加按需检测与未知结果测试**

```rust
#[test]
fn on_demand_detection_touches_only_requested_tools() {
    let harness = DetectionHarness::new(PlatformId::WindowsX64)
        .with_existing_observation(healthy_git("2.47.1"))
        .with_broken_tool(ToolId::CodexCli);
    let run_id = harness
        .service
        .start(Some(vec![ToolId::CodexCli]))
        .unwrap();
    harness.wait_until_completed(&run_id);

    let snapshot = harness.store.snapshot();
    assert_eq!(observation(&snapshot, ToolId::Git).version.as_deref(), Some("2.47.1"));
    assert_eq!(
        observation(&snapshot, ToolId::CodexCli).state,
        ObservedToolState::PresentBroken
    );
    assert!(observation_opt(&snapshot, ToolId::ClaudeCode).is_none());
}
```

- [ ] **Step 5: 验证与提交**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml application::detection
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
git add apps/desktop/src-tauri/src/application
git commit -m "feat: coordinate read-only detection runs"
```

Expected: PASS；selection、run status 和 observations 保持不同字段。

---

### Task 8：接通窄 Tauri commands 与可丢失 detection.changed 事件

**Files:**
- Create: `apps/desktop/src-tauri/src/api/events.rs`
- Modify: `apps/desktop/src-tauri/src/api/mod.rs:1`
- Modify: `apps/desktop/src-tauri/src/api/commands.rs:1-33`
- Modify: `apps/desktop/src-tauri/src/lib.rs:1-22`
- Modify: `apps/desktop/src-tauri/tests/security_configuration.rs:1-86`
- Test: `apps/desktop/src-tauri/src/api/commands.rs`
- Test: `apps/desktop/src-tauri/src/api/events.rs`

**Interfaces:**
- Consumes: `BootstrapService`、`DetectionService`。
- Produces: Tauri `bootstrap`、`detect_tools`，event name `detection.changed`。

- [ ] **Step 1: 写 command core RED**

```rust
#[test]
fn bootstrap_core_returns_the_current_snapshot() {
    let services = fake_app_services(PlatformId::WindowsX64);
    let snapshot = bootstrap_inner(&services.bootstrap);
    assert_eq!(snapshot.platform, PlatformId::WindowsX64);
    assert_eq!(snapshot.schema_version, 2);
    assert_eq!(snapshot.snapshot_version, 1);
}

#[test]
fn detect_tools_core_accepts_full_or_subset_and_rejects_slice_two_tools() {
    let services = fake_app_services(PlatformId::MacosArm64);
    assert!(detect_tools_inner(&services.detection, None).is_ok());

    let other = fake_app_services(PlatformId::MacosArm64);
    assert!(detect_tools_inner(
        &other.detection,
        Some(vec![ToolId::Nodejs])
    ).is_err());
}
```

- [ ] **Step 2: 运行 RED**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap_core_returns_the_current_snapshot
```

Expected: FAIL，因为仍存在旧 `get_bootstrap_state` API。

- [ ] **Step 3: 实现薄 command GREEN**

```rust
#[tauri::command]
pub fn bootstrap(service: tauri::State<'_, BootstrapService>) -> AppSnapshot;

#[tauri::command]
pub fn detect_tools(
    tool_ids: Option<Vec<ToolId>>,
    service: tauri::State<'_, DetectionService>,
) -> Result<String, CommandError>;
```

稳定错误映射：

| 内部错误 | code | retryable |
|---|---|---:|
| empty/duplicate/unsupported selection | `invalidToolSelection` | false |
| active run | `detectionAlreadyRunning` | true |
| counter/store invariant | `internal` | true |

`CommandError` 与 `CommandErrorCode` 必须从 `domain/contracts.rs` 使用并由 ts-rs 生成，不得在 API 层手写第二份错误协议。错误不得包含 stdout/stderr、绝对用户路径或 Rust debug 字符串。删除 `get_bootstrap_state`，不保留双 API。

- [ ] **Step 4: 实现 Tauri event sink**

`TauriDetectionEventSink` 只调用：

```rust
app_handle.emit("detection.changed", event)
```

它实现 `DetectionEventSink` 并把 Tauri emit error 映射为不含原始 payload 的 `EventPublishError`。事件不携带 observation；前端收到后必须 bootstrap。

- [ ] **Step 5: 在 setup 中组装唯一服务图**

`lib.rs` 必须在 `tauri::Builder::setup` 内构造一次：

```text
NativePlatformAdapter
NativeProcessRunner
SystemClock
built_in_detector_registry
SnapshotStore
TauriDetectionEventSink
BootstrapService
DetectionService
```

manage 两个 service，只注册：

```rust
tauri::generate_handler![
    api::commands::bootstrap,
    api::commands::detect_tools
]
```

不注册 plugin，不开启网络，不启动检测；首次检测由向导明确触发。

- [ ] **Step 6: 锁定 capability/CSP/零遥测 RED→GREEN**

先在 `security_configuration.rs` 增加：

```rust
#[test]
fn slice_one_adds_no_privileged_or_network_surface() {
    let capability = read_json("capabilities/main.json");
    assert_eq!(
        capability["permissions"],
        serde_json::json!([
            "core:app:default",
            "core:event:default",
            "core:window:default"
        ])
    );

    let cargo = read_text("Cargo.toml");
    for forbidden in ["reqwest", "ureq", "opentelemetry", "sentry", "analytics"] {
        assert!(!cargo.to_ascii_lowercase().contains(forbidden));
    }

    let config = read_json("tauri.conf.json");
    assert_eq!(
        config["app"]["security"]["csp"]
            .as_str()
            .unwrap()
            .split(';')
            .map(str::trim)
            .find(|part| part.starts_with("connect-src")),
        Some("connect-src ipc: http://ipc.localhost")
    );
}
```

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml slice_one_adds_no_privileged_or_network_surface
```

Expected: PASS without changing capability or CSP。若实现引入依赖导致 RED，删除该依赖并用标准库实现；不得放宽测试。

- [ ] **Step 7: 验证与提交**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml api
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test security_configuration
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
git add apps/desktop/src-tauri/src/api apps/desktop/src-tauri/src/lib.rs apps/desktop/src-tauri/tests/security_configuration.rs
git commit -m "feat: expose snapshot detection IPC"
```

Expected: PASS；Tauri surface 只有两个 commands 和一个 core event。

---

### Task 9：前端 API 与 snapshot 同步状态机处理重复、丢失和旧响应

**Files:**
- Modify: `apps/desktop/src/shared/api/client.ts:1-7`
- Create: `apps/desktop/src/shared/api/client.test.ts`
- Create: `apps/desktop/src/features/detection/useDetectionSnapshot.ts`
- Create: `apps/desktop/src/features/detection/useDetectionSnapshot.test.tsx`
- Test: same files

**Interfaces:**
- Consumes: generated `AppSnapshot`、`DetectionEventEnvelope`、`ToolId`。
- Produces:
  - `bootstrap(): Promise<AppSnapshot>`
  - `detectTools(toolIds?: ToolId[]): Promise<string>`
  - `subscribeDetectionChanged(listener): Promise<UnlistenFn>`
  - `useDetectionSnapshot(api?)`

- [ ] **Step 1: 写 API client RED**

```ts
test("uses only the two narrow commands and stable event name", async () => {
  mockedInvoke
    .mockResolvedValueOnce(snapshot({ snapshotVersion: 1, lastEventSequence: 0 }))
    .mockResolvedValueOnce("run-1");
  mockedListen.mockResolvedValue(() => undefined);

  await bootstrap();
  await detectTools();
  await detectTools(["git"]);
  await subscribeDetectionChanged(() => undefined);

  expect(mockedInvoke).toHaveBeenNthCalledWith(1, "bootstrap");
  expect(mockedInvoke).toHaveBeenNthCalledWith(2, "detect_tools", { toolIds: undefined });
  expect(mockedInvoke).toHaveBeenNthCalledWith(3, "detect_tools", { toolIds: ["git"] });
  expect(mockedListen).toHaveBeenCalledWith("detection.changed", expect.any(Function));
});
```

Run:

```bash
npm run test --workspace @code-ready/desktop -- src/shared/api/client.test.ts
```

Expected: FAIL because client still exports `getBootstrapState` and uses `get_bootstrap_state`。

- [ ] **Step 2: 实现最小 API GREEN**

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  AppSnapshot,
  CommandError,
  CommandErrorCode,
  DetectionEventEnvelope,
  ToolId,
} from "./generated";

export function bootstrap(): Promise<AppSnapshot> {
  return invoke<AppSnapshot>("bootstrap");
}

export function detectTools(toolIds?: ToolId[]): Promise<string> {
  return invoke<string>("detect_tools", { toolIds });
}

export function subscribeDetectionChanged(
  listener: (event: DetectionEventEnvelope) => void,
): Promise<UnlistenFn> {
  return listen<DetectionEventEnvelope>("detection.changed", ({ payload }) => {
    listener(payload);
  });
}

export function commandErrorCode(error: unknown): CommandErrorCode {
  if (
    typeof error === "object"
    && error !== null
    && "code" in error
    && ["invalidToolSelection", "detectionAlreadyRunning", "internal"]
      .includes(String(error.code))
  ) {
    return (error as CommandError).code;
  }
  return "internal";
}
```

不得导出手写 snapshot/event 类型，不 runtime 拼接 command/event 名。

- [ ] **Step 3: 写 sequence/reload/focus RED**

为 hook 注入如下接口，生产默认使用真实 client：

```ts
export interface DetectionApi {
  bootstrap: () => Promise<AppSnapshot>;
  detectTools: (toolIds?: ToolId[]) => Promise<string>;
  subscribeDetectionChanged: (
    listener: (event: DetectionEventEnvelope) => void,
  ) => Promise<() => void>;
}
```

测试组件只显示 hook 的 `phase`、snapshotVersion、lastEventSequence、syncWarning。覆盖：

```ts
test("subscribes before bootstrap and applies the returned snapshot", async () => {
  const api = fakeApi()
    .withSnapshot(snapshot({ snapshotVersion: 1, lastEventSequence: 0 }));
  render(<HookProbe api={api} />);

  expect(await screen.findByText("ready:1:0")).toBeInTheDocument();
  expect(api.calls).toEqual(["subscribe", "bootstrap"]);
});

test("coalesces duplicate events and refreshes from snapshot truth", async () => {
  const api = fakeApi()
    .withSnapshots(
      snapshot({ snapshotVersion: 1, lastEventSequence: 0 }),
      snapshot({ snapshotVersion: 3, lastEventSequence: 2 }),
    );
  render(<HookProbe api={api} />);
  await screen.findByText("ready:1:0");

  act(() => {
    api.emit(event({ sequence: 2, snapshotVersion: 3 }));
    api.emit(event({ sequence: 2, snapshotVersion: 3 }));
  });

  expect(await screen.findByText("ready:3:2")).toBeInTheDocument();
  expect(api.bootstrapCallCount()).toBe(2);
});

test("a sequence gap causes bootstrap instead of event replay", async () => {
  const api = fakeApi()
    .withSnapshots(
      snapshot({ snapshotVersion: 4, lastEventSequence: 3 }),
      snapshot({ snapshotVersion: 8, lastEventSequence: 7 }),
    );
  render(<HookProbe api={api} />);
  await screen.findByText("ready:4:3");

  act(() => api.emit(event({ sequence: 7, snapshotVersion: 8 })));

  expect(await screen.findByText("ready:8:7")).toBeInTheDocument();
  expect(api.bootstrapCallCount()).toBe(2);
});

test("never applies a slower older snapshot over a newer snapshot", async () => {
  const first = deferred<AppSnapshot>();
  const second = deferred<AppSnapshot>();
  const api = fakeApi().withDeferredSnapshots(first, second);
  render(<HookProbe api={api} />);

  act(() => window.dispatchEvent(new Event("focus")));
  second.resolve(snapshot({ snapshotVersion: 5, lastEventSequence: 4 }));
  expect(await screen.findByText("ready:5:4")).toBeInTheDocument();
  first.resolve(snapshot({ snapshotVersion: 1, lastEventSequence: 0 }));

  expect(screen.getByText("ready:5:4")).toBeInTheDocument();
});

test("window focus reloads the snapshot even when no event arrived", async () => {
  const api = fakeApi().withSnapshots(
    snapshot({ snapshotVersion: 1, lastEventSequence: 0 }),
    snapshot({ snapshotVersion: 2, lastEventSequence: 1 }),
  );
  render(<HookProbe api={api} />);
  await screen.findByText("ready:1:0");

  act(() => window.dispatchEvent(new Event("focus")));

  expect(await screen.findByText("ready:2:1")).toBeInTheDocument();
});
```

- [ ] **Step 4: 运行 RED**

```bash
npm run test --workspace @code-ready/desktop -- src/features/detection/useDetectionSnapshot.test.tsx
```

Expected: FAIL because hook is absent。

- [ ] **Step 5: 实现状态机 GREEN**

返回接口固定为：

```ts
export type SnapshotPhase = "loading" | "ready" | "bootstrapError";
export type SyncWarning = null | "eventUnavailable" | "refreshFailed";

export interface DetectionSnapshotState {
  phase: SnapshotPhase;
  snapshot: AppSnapshot | null;
  syncWarning: SyncWarning;
  detectionStartError: CommandErrorCode | null;
  refresh: () => Promise<void>;
  startDetection: (toolIds?: ToolId[]) => Promise<void>;
}
```

实现细则：

1. effect 先 await subscribe，再 bootstrap，避免 subscribe 前的事件窗口。
2. subscribe 失败时设置 `eventUnavailable`，但仍 bootstrap；UI 可通过 focus/manual refresh 恢复。
3. 每个 bootstrap response 只有 `snapshotVersion >= 当前 snapshotVersion` 才能应用。
4. event.sequence `<= snapshot.lastEventSequence` 立即忽略。
5. event.sequence 更大时只触发 bootstrap，不应用 event payload 中的事实。
6. 同一时刻最多一个 refresh promise；期间记录最高 seen sequence。完成后若最高 seen sequence 仍大于返回 snapshot.lastEventSequence，再做一次 refresh。
7. sequence 缺口和连续事件使用同一 bootstrap 路径；缺口不尝试 replay。
8. focus 事件总是调用 refresh；卸载时移除 focus listener 和 Tauri unlisten。
9. bootstrap 初次失败为 `bootstrapError`；已有 snapshot 后 refresh 失败保留旧 snapshot 并显示 `refreshFailed`。
10. `startDetection` 只调用 command；运行事实等待后端 event/snapshot，不在前端伪造 Running。
11. command reject 只映射稳定 error code；未知 reject 统一 internal，不显示 Rust/debug 文本。

- [ ] **Step 6: 增加 listener cleanup 和错误恢复测试**

覆盖：

- unmount 调用一次 unlisten；
- event subscription reject 后仍显示 snapshot；
- 初始 bootstrap reject 后 `refresh()` 成功进入 ready；
- 有旧 snapshot 时 refresh reject 不清空事实；
- `startDetection(["git"])` 精确透传，reject 时 observation 不变。

Run:

```bash
npm run test --workspace @code-ready/desktop -- src/features/detection/useDetectionSnapshot.test.tsx
npm run typecheck
npm run lint
```

Expected: PASS。

- [ ] **Step 7: 提交**

```bash
git add apps/desktop/src/shared/api/client.ts apps/desktop/src/shared/api/client.test.ts apps/desktop/src/features/detection
git commit -m "feat: synchronize detection snapshots"
```

---

### Task 10：用 i18n key 完成进程内首次向导

**Files:**
- Create: `apps/desktop/src/features/detection/presentation.ts`
- Create: `apps/desktop/src/features/onboarding/Onboarding.tsx`
- Create: `apps/desktop/src/features/onboarding/Onboarding.test.tsx`
- Modify: `apps/desktop/src/shared/i18n/zh-CN.ts:1-15`
- Create: `apps/desktop/src/shared/i18n/zh-CN.test.ts`
- Test: same files

**Interfaces:**
- Consumes: snapshot、`startDetection`。
- Produces: welcome → detecting/explanation → `onComplete()` 的进程内向导；全部用户文案来自 zh-CN resource。

- [ ] **Step 1: 写 presentation/i18n RED**

```ts
test.each([
  ["absent", "status.absent"],
  ["presentHealthy", "status.presentHealthy"],
  ["presentPathIssue", "status.presentPathIssue"],
  ["presentBroken", "status.presentBroken"],
  ["unknown", "status.unknown"],
] as const)("maps fact %s without inspecting version state", (state, key) => {
  expect(factMessageKey(state)).toBe(key);
});

test.each([
  ["current", "version.current"],
  ["outdated", "version.outdated"],
  ["newerThanKnown", "version.newerThanKnown"],
  ["notComparable", "version.notComparable"],
  ["unknown", "version.unknown"],
] as const)("maps version status %s separately", (state, key) => {
  expect(versionMessageKey(state)).toBe(key);
});

test("translates every registry tool label used by slice one", () => {
  for (const key of [
    "tools.git.name",
    "tools.claudeCode.name",
    "tools.codexCli.name",
  ]) {
    expect(toolLabel(key)).not.toEqual(key);
  }
});
```

- [ ] **Step 2: 运行 RED**

```bash
npm run test --workspace @code-ready/desktop -- src/shared/i18n/zh-CN.test.ts
```

Expected: FAIL because Slice 1 keys and presentation mapper are absent。

- [ ] **Step 3: 实现 typed resource GREEN**

资源至少包含这些稳定 key 和中文：

```ts
export const messages = {
  "app.title": "Code-Ready V2",
  "privacy.zeroTelemetry": "检测只在当前设备上进行，不上传遥测或命令输出。",
  "onboarding.welcome.title": "先看看这台电脑的开发环境",
  "onboarding.welcome.body": "Code-Ready 会只读检查 Git、Claude Code 和 Codex CLI，不会安装、升级或修复任何工具。",
  "onboarding.start": "开始检测",
  "onboarding.detecting.title": "正在检测开发工具",
  "onboarding.explain.title": "检测完成",
  "onboarding.enterStatusCenter": "进入状态中心",
  "detection.running": "检测正在进行…",
  "detection.runFailed": "检测任务意外停止，工具卡片仍显示最近一次已知事实。",
  "detection.startError": "暂时无法开始检测，请重试。",
  "sync.eventUnavailable": "实时更新暂不可用；重新聚焦窗口或手动刷新可读取最新状态。",
  "sync.refreshFailed": "暂时无法刷新，下面仍显示最近一次已知状态。",
  "status.absent": "未检测到",
  "status.presentHealthy": "可以正常运行",
  "status.presentPathIssue": "已安装，但当前环境路径没有指向它",
  "status.presentBroken": "已找到，但无法正常完成版本检查",
  "status.unknown": "暂时无法判断",
  "version.current": "版本符合当前基线",
  "version.outdated": "版本低于支持基线",
  "version.newerThanKnown": "版本高于当前已知范围",
  "version.notComparable": "已检测到版本，但本版本暂不判断新旧",
  "version.unknown": "未能读取版本",
  "tools.git.name": "Git",
  "tools.claudeCode.name": "Claude Code",
  "tools.codexCli.name": "Codex CLI",
  "actions.retry": "重试",
  "actions.refresh": "刷新状态",
  "actions.redetectAll": "重新检测全部",
  "actions.redetectOne": "重新检测此工具",
} as const;

export type MessageKey = keyof typeof messages;
export function message(key: MessageKey): string;
export function toolLabel(labelKey: string): string;
export function formatObservedVersion(version: string): string;
```

`toolLabel` 用显式 map；未知 key 返回“未知工具”，不得直接把 key 暴露给用户。版本只做本地字符串格式化，不请求 locale/network。

- [ ] **Step 4: 写向导 RED**

```ts
test("runs the approved process-local onboarding flow", async () => {
  const startDetection = vi.fn().mockResolvedValue(undefined);
  const { rerender } = render(
    <Onboarding
      snapshot={snapshot({ detectionRun: null, observations: [] })}
      startDetection={startDetection}
      onComplete={vi.fn()}
    />,
  );

  expect(screen.getByRole("heading", { name: "先看看这台电脑的开发环境" }))
    .toBeInTheDocument();
  expect(screen.getByText(/不会安装、升级或修复/)).toBeInTheDocument();
  expect(screen.getByText(/不上传遥测/)).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "开始检测" }));
  expect(startDetection).toHaveBeenCalledWith(undefined);

  rerender(
    <Onboarding
      snapshot={snapshot({ detectionRun: runningRun(), observations: [] })}
      startDetection={startDetection}
      onComplete={vi.fn()}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent("检测正在进行");

  const onComplete = vi.fn();
  rerender(
    <Onboarding
      snapshot={snapshot({ detectionRun: completedRun(), observations: threeFacts() })}
      startDetection={startDetection}
      onComplete={onComplete}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "进入状态中心" }));
  expect(onComplete).toHaveBeenCalledOnce();
});
```

- [ ] **Step 5: 运行 RED**

```bash
npm run test --workspace @code-ready/desktop -- src/features/onboarding/Onboarding.test.tsx
```

Expected: FAIL because component is absent。

- [ ] **Step 6: 实现最小向导 GREEN**

状态只由 snapshot + 当前进程 UI step 推导：

```ts
type OnboardingStep = "welcome" | "detection" | "explanation";
```

- 初始 snapshot 没有 detectionRun 时为 welcome。
- 点击开始调用 `startDetection(undefined)`，等待后端 snapshot 进入 Running；button 在 promise pending 时 disabled。
- Running 时为 detection，使用 `role="status"`、`aria-live="polite"`，但不把任一 tool fact 改成 checking。
- Completed 且至少有本轮请求工具对应 observation 时为 explanation。
- Failed 时保留已有 observation，显示 `detection.runFailed` 的 alert 和“重试”；不得把未完成工具伪造成 unknown/absent。
- 每个 observation 同时渲染事实文案和独立版本文案；`notComparable` 不使用“最新/过旧”字样。
- start error 显示 `role="alert"` 与重试 button。
- 不渲染安装、修复、升级、权限、登录、代理或日志控件。
- 完成 button 只调用 `onComplete`，不写磁盘/localStorage。

- [ ] **Step 7: 键盘和语义验证**

测试：

- heading 层级只有一个 h1，步骤标题为 h2；
- 所有 button 有中文 accessible name；
- error 使用 alert，running 使用 status；
- Tab 顺序为主操作后辅助重试；
- 不存在 `/安装|升级|修复|管理员|登录|API Key|代理|日志/` 对应 button。

Run:

```bash
npm run test --workspace @code-ready/desktop -- src/features/onboarding/Onboarding.test.tsx
npm run test --workspace @code-ready/desktop -- src/shared/i18n/zh-CN.test.ts
```

Expected: PASS。

- [ ] **Step 8: 提交**

```bash
git add apps/desktop/src/features/detection/presentation.ts apps/desktop/src/features/onboarding apps/desktop/src/shared/i18n
git commit -m "feat: add process-local first-run guide"
```

---

### Task 11：只读状态中心呈现五种事实、版本和按需/全量重检

**Files:**
- Create: `apps/desktop/src/features/dashboard/StatusCenter.tsx`
- Create: `apps/desktop/src/features/dashboard/StatusCenter.test.tsx`
- Modify: `apps/desktop/src/app/App.tsx:1-79`
- Modify: `apps/desktop/src/app/App.test.tsx:1-54`
- Modify: `apps/desktop/src/app/styles.css:1-83`
- Test: same files

**Interfaces:**
- Consumes: `DetectionSnapshotState`、onboarding completion callback。
- Produces: 相同双平台 UX 的只读状态中心。

- [ ] **Step 1: 写五状态组合 RED**

```ts
test.each([
  ["absent", "未检测到"],
  ["presentHealthy", "可以正常运行"],
  ["presentPathIssue", "已安装，但当前环境路径没有指向它"],
  ["presentBroken", "已找到，但无法正常完成版本检查"],
  ["unknown", "暂时无法判断"],
] as const)("renders %s as a fact, not a task state", (state, label) => {
  render(
    <StatusCenter
      snapshot={snapshotWithObservation(observation({ state }))}
      startDetection={vi.fn()}
      refresh={vi.fn()}
      syncWarning={null}
      detectionStartError={null}
    />,
  );

  expect(screen.getByText(label)).toBeInTheDocument();
  expect(screen.queryByText(/downloading|executing|cancelled/)).not.toBeInTheDocument();
});

test("shows parsed version and notComparable independently", () => {
  renderStatus(observation({
    state: "presentHealthy",
    version: "0.138.0",
    versionStatus: "notComparable",
  }));
  expect(screen.getByText("版本 0.138.0")).toBeInTheDocument();
  expect(screen.getByText("已检测到版本，但本版本暂不判断新旧")).toBeInTheDocument();
  expect(screen.queryByText(/最新|需要升级|过旧/)).not.toBeInTheDocument();
});
```

- [ ] **Step 2: 运行 RED**

```bash
npm run test --workspace @code-ready/desktop -- src/features/dashboard/StatusCenter.test.tsx
```

Expected: FAIL because status center is absent。

- [ ] **Step 3: 实现只读状态中心 GREEN**

布局固定：

- 页面 h1 “开发环境状态”。
- 顶部平台 label 与“只读检测”说明。
- 分组“需要处理”含 absent/pathIssue/broken/unknown；“正常可用”含 healthy。
- 每张卡显示工具名、事实 badge、版本行、版本状态解释、脱敏路径详情。
- 检测 run Running 时顶部显示独立进度，不覆盖卡片最近事实。
- 每卡“重新检测此工具”调用 `startDetection([tool.id])`。
- 顶部“重新检测全部”调用 `startDetection(undefined)`。
- “刷新状态”只调用 bootstrap refresh，不启动进程。
- active run 时重检 buttons disabled；刷新仍可用。
- 不提供任何安装/修复链接或外部 URL。

snapshot 中 5 项产品 registry 只渲染存在 observation 的三项 Slice 1 工具；未检测的 Slice 1 工具显示“尚未检测”的 presentation-only 空态，不构造 `unknown` observation。Winget/Nodejs 不出现在 Slice 1 UI。

- [ ] **Step 4: 写交互与双平台等价 RED**

```ts
test("starts one-tool and full read-only detection", () => {
  const startDetection = vi.fn();
  renderStatusCenter({ startDetection });

  fireEvent.click(screen.getByRole("button", { name: "重新检测 Git" }));
  expect(startDetection).toHaveBeenCalledWith(["git"]);

  fireEvent.click(screen.getByRole("button", { name: "重新检测全部" }));
  expect(startDetection).toHaveBeenCalledWith(undefined);
});

test.each([
  ["windowsX64", "Windows 11 x64"],
  ["macosArm64", "macOS Apple Silicon"],
] as const)("keeps identical controls and state semantics on %s", (platform, label) => {
  renderStatusCenter({ snapshot: completeSnapshot(platform) });
  expect(screen.getByText(label)).toBeInTheDocument();
  expect(screen.getAllByRole("button").map((button) => button.textContent)).toEqual([
    "刷新状态",
    "重新检测全部",
    "重新检测 Git",
    "重新检测 Claude Code",
    "重新检测 Codex CLI",
  ]);
});
```

- [ ] **Step 5: App 组合 RED**

改写 `App.test.tsx`，mock `useDetectionSnapshot` 而不是旧 client，覆盖：

1. loading 显示“正在读取当前设备…”。
2. bootstrapError 显示中文错误和重试。
3. 当前进程第一次、没有 run 时显示 Onboarding。
4. completed snapshot 初次 mount 直接进入 StatusCenter，满足 UI reload 后恢复。
5. 当前进程点击完成后进入 StatusCenter。
6. sync warning 是非阻塞 banner。

Run:

```bash
npm run test --workspace @code-ready/desktop -- src/app/App.test.tsx
```

Expected: FAIL because App still使用 Slice 0 client/summary。

- [ ] **Step 6: 实现 App GREEN**

`App.tsx` 只做组合。把 ready 分支拆成独立组件，使它第一次 mount 时根据首个可用 snapshot 决定本进程 UI 模式：

```tsx
export default function App() {
  const detection = useDetectionSnapshot();

  if (detection.phase === "loading") return <LoadingScreen />;
  if (detection.phase === "bootstrapError") {
    return <BootstrapErrorScreen onRetry={detection.refresh} />;
  }

  return <ReadyApp detection={detection} />;
}

function ReadyApp({ detection }: { detection: ReadyDetectionSnapshotState }) {
  const snapshot = detection.snapshot;
  const [mode, setMode] = useState<"onboarding" | "dashboard">(
    () => snapshot.detectionRun?.status === "completed"
      ? "dashboard"
      : "onboarding",
  );

  if (mode === "onboarding") {
    return (
      <Onboarding
        snapshot={snapshot}
        startDetection={detection.startDetection}
        onComplete={() => setMode("dashboard")}
      />
    );
  }

  return <StatusCenter {...statusCenterProps(snapshot, detection)} />;
}
```

若当前进程从 welcome 开始，`ReadyApp` 的 mode 保持 onboarding，run completed 后仍先显示 explanation，用户点击后进状态中心；若 UI reload 时首个 snapshot 已有 completed run，新 `ReadyApp` 直接选择 dashboard。不得写 localStorage。

- [ ] **Step 7: 样式与系统缩放**

重写 `styles.css` 为 feature class，保留：

- 320 px 最小宽度无水平溢出；
- 200% 浏览器/系统缩放时单列；
- `:focus-visible` 2 px 高对比 outline；
- 状态不只靠颜色，始终有文本；
- `prefers-reduced-motion` 下禁用非必要 transition；
- 字体继续用系统栈，不加载远程 font/image；
- 卡片路径使用 `overflow-wrap: anywhere`。

不得加入 inline script、远程资源或 CSP 新 source。

- [ ] **Step 8: 验证与提交**

```bash
npm run test --workspace @code-ready/desktop -- src/features/dashboard/StatusCenter.test.tsx
npm run test --workspace @code-ready/desktop -- src/app/App.test.tsx
npm run typecheck
npm run lint
npm run build
git add apps/desktop/src/app apps/desktop/src/features/dashboard
git commit -m "feat: add read-only status center"
```

Expected: PASS，Vite 生产构建成功。

---

### Task 12：Fake 双平台纵向测试、CI 语义和 Slice 1 收口

**Files:**
- Create: `apps/desktop/src-tauri/tests/detection_flow.rs`
- Modify: `apps/desktop/src-tauri/tests/security_configuration.rs`
- Modify: `docs/testing/v2-platform-matrix.md`
- Test: Rust integration、React suites、root checks

**Interfaces:**
- Consumes: 完整 Rust service graph 与 React snapshot fixtures。
- Produces: hosted compile/test 门禁和明确的未完成干净机验收记录。

- [ ] **Step 1: 写 FakePlatformAdapter/FakeProcessRunner 纵向 RED**

```rust
#[test]
fn windows_and_macos_fake_flows_produce_identical_fact_semantics() {
    for platform in [PlatformId::WindowsX64, PlatformId::MacosArm64] {
        let harness = EndToEndHarness::new(platform)
            .with_path_native(ToolId::Git, "git version 2.47.1")
            .with_known_native(ToolId::ClaudeCode, "2.1.89 (Claude Code)")
            .without_candidate(ToolId::CodexCli);

        let initial = harness.bootstrap();
        assert!(initial.observations.is_empty());
        assert!(initial.detection_run.is_none());

        let run_id = harness.detect_all().unwrap();
        harness.wait_until_completed(&run_id);
        let final_snapshot = harness.bootstrap();

        assert_eq!(
            facts(&final_snapshot),
            vec![
                (ToolId::Git, ObservedToolState::PresentHealthy, VersionStatus::NotComparable),
                (
                    ToolId::ClaudeCode,
                    ObservedToolState::PresentPathIssue,
                    VersionStatus::NotComparable,
                ),
                (ToolId::CodexCli, ObservedToolState::Absent, VersionStatus::Unknown),
            ]
        );
        assert_eq!(final_snapshot.last_event_sequence, 5);
        assert_eq!(final_snapshot.snapshot_version, 6);
        assert_eq!(
            final_snapshot.detection_run.unwrap().status,
            DetectionRunStatus::Completed
        );
    }
}

#[test]
fn detection_flow_is_read_only_and_never_requests_privilege_or_network() {
    let harness = EndToEndHarness::new(PlatformId::WindowsX64).with_all_absent();
    let run_id = harness.detect_all().unwrap();
    harness.wait_until_completed(&run_id);

    assert!(harness.process_requests().is_empty());
    assert_eq!(harness.privilege_requests(), 0);
    assert_eq!(harness.network_requests(), 0);
}
```

`privilege_requests/network_requests` 是 harness 中恒为 0 的 capability counters，不得为 production 添加 privilege/network API。

- [ ] **Step 2: 运行 RED**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test detection_flow
```

Expected: compile FAIL，因为 integration harness 尚未接通 public service constructors，或行为尚未完整。

- [ ] **Step 3: 最小暴露 testable constructors 并跑 GREEN**

只把执行所需构造器设为 `pub`；不得导出内部 mutex、测试注入 setter 或修改事实的后门。integration harness 使用 public traits 注入 fake，不通过 Tauri。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test detection_flow
```

Expected: 两个平台 fixture PASS；相同输入事实只差 platform ID/展示路径。

- [ ] **Step 4: 增加契约与源码静态门禁**

在 `security_configuration.rs` 增加以下检查：

1. `lib.rs` 的 handler 文本包含且只包含 `bootstrap`、`detect_tools`。
2. Tauri event 固定为 `detection.changed`。
3. capabilities 仍是 Slice 0 精确 allowlist。
4. CSP 完全保持 Slice 0 字符串。
5. `Cargo.toml` 无网络、遥测、SQLite、async runtime、shell/process plugin 依赖。
6. `package.json` 无 i18n 网络 loader、telemetry、analytics 依赖。
7. `tools/claude.rs`、`tools/codex.rs` 无 Node/npm/shell token。
8. generated 目录不存在 `BootstrapState.ts`，存在 `AppSnapshot.ts` 与 `DetectionEventEnvelope.ts`。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test security_configuration
```

Expected: PASS。不得通过删除断言或扩大 allowlist 修绿。

- [ ] **Step 5: 更新平台门禁文档**

在 `docs/testing/v2-platform-matrix.md` 新增 Slice 1 段落，逐字表达：

- `windows-2025` 是 hosted Windows Server x64；只证明 compile/test 和 FakePlatformAdapter 契约，不等于 Windows 11 x64 普通用户干净机检测。
- `macos-15` arm64 只证明 Apple Silicon hosted compile/test 和 fake contract；`MACOSX_DEPLOYMENT_TARGET=13.0` 不等于 macOS 13 干净机验收。
- Slice 1 真实验收仍需：
  - Windows 11 x64 普通用户，三工具分别 absent/healthy/PATH issue/broken；
  - macOS 13+ Apple Silicon 普通用户，同一矩阵；
  - macOS 未装 CLT 时确认检测不会弹出 Apple 安装对话框；
  - 两个平台确认 `--version` timeout 后 UI 可恢复；
  - UI reload、窗口重新聚焦、重复事件、sequence 缺口；
  - 进程监控确认无 Node/npm、无提权、无网络、无遥测。
- 在上述机器验收完成前，只能报告“hosted compile/test coverage”，不能写“Windows 11/macOS 13 验收通过”。

`.github/workflows/ci.yml` 已运行根 `npm run check` 和 Tauri build，现有 root scripts 会自动收集新增 Rust integration 与 Vitest 文件，因此本切片不修改 workflow。不得增加伪造 OS 版本的 label 或成功声明。

- [ ] **Step 6: 运行 binding drift 红灯演练**

在执行工作树中临时给 `AppSnapshot.ts` 末尾加一个空行（此临时动作不提交），运行：

```bash
npm run check:bindings
```

Expected: 非零并显示 generated diff。随后运行：

```bash
npm run generate:bindings
npm run check:bindings
```

Expected: 退出码 0。执行记录必须注明红灯确实出现；最终 diff 不含人为破坏。

- [ ] **Step 7: 运行完整本地验证**

```bash
git status --short
npm ci
npm run check
npm run build
npm run tauri:build
git diff --check
```

Expected:

- TypeScript typecheck、ESLint、全部 Vitest PASS。
- Rust fmt、clippy `-D warnings`、unit/integration tests PASS。
- binding drift PASS。
- Vite 和当前宿主的 unsigned Tauri app build PASS。
- `git diff --check` 无输出。
- status 只含本任务预期文件；无 `dist`、`target`、日志、snapshot dump 或 fixture 输出。

- [ ] **Step 8: 运行范围与安全人工审查**

```bash
rg -n -i "install|upgrade|repair|download|proxy|sqlite|legacy|migration|helper|uac|sudo|authorization|updater|telemetry|analytics" \
  apps/desktop/src-tauri/src \
  apps/desktop/src \
  apps/desktop/src-tauri/Cargo.toml \
  apps/desktop/package.json
```

Expected: 仅允许：

- Slice 0 `ToolCapability` 和工具产品定义中的 install/upgrade/repair 枚举；
- 简体中文“不会安装、升级或修复”的只读承诺；
- 安全测试的禁止词 fixture。

任何 production action、依赖、command、button 或网络 endpoint 命中都必须删除。

继续检查：

```bash
rg -n -i "node(js)?|npm|\\.cmd|\\.ps1|powershell|cmd\\.exe|/bin/sh" \
  apps/desktop/src-tauri/src/tools/claude.rs \
  apps/desktop/src-tauri/src/tools/codex.rs

git grep -n "type AppSnapshot\\|type ObservedToolState\\|type VersionStatus" \
  -- apps/desktop/src ':!apps/desktop/src/shared/api/generated'

git diff -- apps/desktop/src-tauri/capabilities/main.json apps/desktop/src-tauri/tauri.conf.json
```

Expected: 三条均无输出。第三条无 diff 证明 CSP/capability 未扩张。

- [ ] **Step 9: 请求代码审查**

使用 `requesting-code-review` 对规范、本文、Slice 1 全部 commits 和以下高风险点做独立审查：

- 工具事实/版本/运行状态是否混合；
- 原生二进制识别是否会误执行 Node/npm launcher；
- macOS Git 是否可能触发安装 UI；
- timeout 是否回收进程和 pipe；
- snapshot/sequence 是否在线程竞争下单调；
- event 丢失是否仍可由 bootstrap/focus 恢复；
- React 是否可能应用旧 snapshot；
- capability/CSP/零遥测是否保持。

P0/P1 必须在收口前修复并重新运行相关 RED→GREEN；低优先级问题需记录明确处置。

- [ ] **Step 10: 提交 Slice 1 收口**

```bash
git add apps/desktop/src-tauri/tests/detection_flow.rs apps/desktop/src-tauri/tests/security_configuration.rs docs/testing/v2-platform-matrix.md
git commit -m "test: verify Slice 1 detection flow"
git status --short --branch
```

Expected: 工作树干净；实现分支仍基于 `codex/v2-rebuild`，没有 push/PR。

---

## Slice 1 测试矩阵

### Rust tool detector matrix（每个平台、每个工具）

| Case | Candidate | Process | Expected fact | Expected version |
|---|---|---|---|---|
| PATH native + parseable | PATH native | exit 0 | healthy | parsed + notComparable |
| PATH native + unparseable | PATH native | exit 0 | healthy | none + unknown |
| Known native + parseable | known native | exit 0 | pathIssue | parsed + notComparable |
| Script shadows known native | PATH nonnative + known native | known exit 0 | pathIssue | parsed + notComparable |
| Script only | PATH nonnative | not executed | broken | none + unknown |
| No candidates | none | not run | absent | none + unknown |
| Nonzero | native | code != 0 | broken | none + unknown |
| Signal | native | no exit code | broken | none + unknown |
| Timeout | native | >3 s | broken | none + unknown |
| Permission denied | native | launch failure | broken | none + unknown |
| Candidate disappears | native | not found on spawn | unknown | none + unknown |
| Platform enumeration error | adapter error | not run | unknown | none + unknown |
| macOS Apple shim, no developer dir | Apple shim | xcode-select nonzero | absent | none + unknown |
| macOS Apple shim, valid developer dir | Apple shim | preflight + git exit 0 | healthy | parsed + notComparable |

### Snapshot/event matrix

| Input | Required behavior |
|---|---|
| bootstrap read | no version/sequence increment |
| run begins | snapshot+1, sequence+1, Running only in run field |
| each observation | atomic upsert, snapshot+1, sequence+1 |
| run completes | facts unchanged, Completed only in run field |
| run infrastructure fails | facts unchanged, Failed/Internal only in run field |
| event publish fails | snapshot remains committed |
| duplicate sequence | front ignores/refetch coalesces |
| lower sequence | front ignores |
| gap | front calls bootstrap; no replay |
| stale bootstrap response | front rejects lower snapshotVersion |
| focus without event | front calls bootstrap |
| listener unavailable | current snapshot usable; warning + focus/manual fallback |

### React state matrix

| Snapshot/UI condition | Screen |
|---|---|
| no snapshot | loading |
| initial bootstrap error | retryable full-page error |
| no completed run in current process | welcome |
| run Running | detection step; old facts remain old facts |
| run Failed | detection error + retry; old facts remain old facts |
| run Completed after welcome | explanation |
| user confirms explanation | status center |
| reload with completed run | status center |
| refresh error with old snapshot | status center + nonblocking warning |
| absent/pathIssue/broken/unknown | “需要处理” |
| healthy | “正常可用” |

## Slice 1 完成定义

只有同时满足以下条件，执行者才能报告 Slice 1 本地实现完成：

1. Git、Claude Code、Codex CLI 在两个 FakePlatformAdapter 平台上通过完整事实矩阵。
2. Claude/Codex detector 的 runner request 只有原生绝对路径和 `--version`，且源码 guard 无 Node/npm/shell。
3. Git macOS Apple shim 在 developer tools 缺失时不执行 `/usr/bin/git`，自动测试证明不会请求安装。
4. 每个 observation 的事实状态、版本值、版本状态、检测 run 状态分离。
5. 所有解析成功版本是 `notComparable`；没有 production path 生成 current/outdated/newerThanKnown。
6. snapshotVersion 与 event sequence 单调，bootstrap 只读；event publish 失败不回滚事实。
7. 前端对重复/旧事件去重，对 gap、focus、listener failure 和旧 response 有恢复测试。
8. 进程内向导与只读状态中心全部用户文案来自 zh-CN resource，Windows/macOS 控件和状态语义一致。
9. `npm run check`、`npm run build`、当前宿主 `npm run tauri:build` 本轮新鲜通过。
10. binding drift 红灯演练已实际观察，最终生成目录无漂移。
11. CSP 与 capability 与 Slice 0 无 diff；安全测试证明无网络、遥测、持久化、提权或 Slice 2+ dependency。
12. hosted Windows x64/macOS arm64 CI 只有在 push 后真实运行才能报告结果；未 push 时写“待运行”。
13. Windows 11 x64 与 macOS 13+ Apple Silicon 普通用户干净机验收仍明确为后续人工 gate，不被 hosted CI 冒充。
14. 工作树干净，所有提交只包含 Slice 1 实现、测试和平台门禁文档。

## 明确不属于 Slice 1

- 跨进程向导完成持久化；
- SQLite、配置、代理、日志、诊断导出、旧配置预览/迁移；
- 可信版本下限/最新版本元数据、网络 version check；
- Winget、Node.js/npm 检测；
- 安装、升级、修复、计划、来源、下载、校验；
- UAC、sudo、Authorization Services、helper、进程树取消；
- 登录、API Key、账号状态；
- 应用更新、签名、公证、公开分发；
- Windows 11/macOS 13 干净机已通过声明。

## 计划自审记录

- **规范覆盖：** Slice 1 固定范围分别映射到 Tasks 1–12；进程内首次与 `notComparable` 已按主代理裁决写成批准约束。
- **路径审计：** 所有 Modify 路径在 `70ffb83` 当前树存在；所有 Create 路径位于设计规范已有的 `features/`、`application/`、`platform/`、`tools/` 边界。
- **接口一致：** `AppSnapshot.snapshotVersion/lastEventSequence`、`DetectionEventEnvelope.sequence/snapshotVersion`、`DetectionService.start`、前端 `bootstrap/detectTools/subscribeDetectionChanged` 在生产任务和测试任务中命名一致。
- **范围审计：** 没有配置、网络、日志、持久化、安装、权限或更新 production API；现有 capability/CSP 保持精确不变。
- **命令审计：** 所有命令从仓库根目录可执行；执行环境先满足现有 Node 24/npm 11 与 Rust 1.88+ 约束。测试 filter 均对应本文定义的 test 名或文件。
- **CI 声明审计：** 只描述 hosted compile/test，不声称 Windows 11 或 macOS 13 干净机已验收。
- **占位审计：** 文档不含未定实现项；所有状态、命令、路径、错误、timeout、输出上限和提交边界均已确定。
