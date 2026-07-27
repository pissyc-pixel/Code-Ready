# Code-Ready V2 Slice 2：配置、网络设置、隐私日志、诊断与 V1 预览 Implementation Plan（实施计划）

> **For agentic workers:** REQUIRED SUB-SKILL: Use `executing-plans`, `test-driven-development`, and `verification-before-completion` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. 执行模型固定为 Luna Max；每项行为变更必须先观察到本文指定的 RED，再写最小 GREEN，最后执行 REFACTOR/回归。不得跳过失败证据。

**Goal:** 在 Windows 11 x64 与 macOS 13+ Apple Silicon 上加入版本化安全设置、离线代理/registry 策略、零遥测本地脱敏日志、用户明确触发的诊断导出，以及 Windows V1 固定位置的只读发现/预览；整个 Slice 不安装、不升级、不修复、不卸载、不执行任意工具、不发起网络请求。

**Architecture:** Rust 继续是设置、日志、诊断、V1 预览和 `AppSnapshot` 的唯一事实来源。设置使用单文档、严格 schema、带 SHA-256 完整性字段的 JSON，通过应用私有固定路径、同目录临时文件、同步、原子替换和最近有效备份实现崩溃恢复；本切片不引入数据库。日志先由类型化事件减少输入面，再写入前递归脱敏，按 UTC 日和 5 MiB 文件滚动，并按 30 天或 100 MiB 先到边界清理；诊断先生成本地暂存包和可审计 manifest，用户确认后才复制到系统 Downloads 固定目录。事件仍只提示 snapshot 变化，React 对 sequence 缺口、重复、旧响应和窗口重新聚焦统一 `bootstrap()`。

**Tech Stack:** 现有 Tauri 2.11、React 19.2.8、TypeScript 6.0.3、Rust 2024/stable、serde、ts-rs 12.0.1；新增最小 Rust 依赖为 `serde_json`、`url`、`regex`、`sha2`、无压缩特性的 `zip`，以及仅用于操作系统安全文件 API 的目标平台依赖。不得新增 JavaScript runtime 依赖、网络客户端、async runtime、数据库或 Tauri plugin。

## Global Constraints

- 基线必须是 `codex/v2-rebuild` 的 `8ada8ca`，或只包含本计划/设计澄清提交的直接后继；执行前用 `git rev-parse HEAD` 核对。
- 支持平台固定为 Windows 11 x64 与 macOS 13.0+ Apple Silicon；hosted CI 不得冒充干净机验收。
- 所有配置使用 schema 版本、严格字段校验、SHA-256 完整性校验、原子写入、最近有效备份、损坏隔离/恢复；前端不得提供配置路径。
- 配置、日志、暂存诊断和导出目标必须拒绝符号链接、Windows reparse point、路径穿越、非普通文件和不安全权限；检查与打开必须由同一安全文件边界完成，不能只做 `exists()` 后再普通打开。
- 网络模式只允许 `direct | systemProxy | manualProxy`；手动代理只允许无凭据的 `http://` 或 `https://` authority URL，不支持 SOCKS，不支持代理认证，不允许 path/query/fragment。
- npm registry 策略只允许 `officialOnly | officialThenNpmmirror`。官方固定为 `https://registry.npmjs.org/`；唯一国内回退固定为 `https://registry.npmmirror.com/`。不得接受、保存或传输任意 registry URL。
- “经过验证的 npmmirror”在 Slice 2 只表示编译时 allowlist、HTTPS/host/path 不变量和代码审查通过；实际可达性、下载来源与官方校验值联动仍延期到 Slice 9。本切片不得为“验证”发出网络请求。
- 系统代理模式只保存“后续网络能力读取系统代理”的意图，不修改操作系统；手动代理只保存 Code-Ready 后续受支持下载器可用的值，不注入 Winget 或任意进程环境。
- 零遥测、零自动外联、零自动上传。CSP 必须保持现有精确字符串；capability 必须保持现有精确三项 core allowlist。
- 日志只接受类型化事件和字段；不得记录环境变量、完整命令行、原始 stdout/stderr、HTTP body、完整用户名路径、代理凭据、API Key、cookie、Authorization 或任意用户输入 map。
- 日志保留边界固定为 30 个 UTC 日或 100 MiB 总量，先到即清理；单文件上限固定为 5 MiB，单条编码后记录上限固定为 16 KiB。
- 诊断包必须先展示 manifest，再由用户明确导出；不得自动上传。包内不含原始配置文件、V1 文件、环境变量、主机名、用户名、数据库、任意命令输出或完整代理 URL。
- 诊断导出命令不接受字符串路径，只接受生成 DTO 的 `downloads` 枚举；输出文件名由 Rust 生成。
- Windows V1 来源只允许 `%APPDATA%\ai-coding-installer\config.json`；macOS 明确返回 `noSourceForPlatform`。不得枚举其他目录或接受用户指定 V1 路径。
- V1 只发现和 preview；不得提供 apply/import/migrate command，不修改、不删除、不备份 V1 文件。实际应用延期到后续经单独批准的切片。
- 所有 IPC DTO 由 Rust serde + ts-rs 权威定义；生成目录只由 exporter 写，前端不得手写同名协议。
- `AppSnapshot.schemaVersion` 从 2 升到 3；`snapshotVersion`/`lastEventSequence` 继续单调。事件投递失败不得回滚已提交设置。
- 本切片不得安装、升级、修复、卸载、下载、提权、打开 shell、执行新进程、修改 PATH、修改系统代理、调用 npm 或包管理器。

---

## 依赖最小化与反向审计结论

### 配置存储：选择原子 JSON，不选择 SQLite

Slice 2 的持久状态是一个小型设置文档：语言、网络策略、向导完成状态、revision 和迁移元数据。它没有关系查询、多写者、长事务或安装 journal。SQLite 的 WAL、SQL parser、C ABI、migration/backup API 会扩大依赖和审计面，却没有提供本切片不可替代的收益。因此：

1. 使用固定 schema 的 JSON struct，不使用通用 key/value map。
2. 对除 checksum 字段外的完整文档体计算 SHA-256，用于发现截断/损坏；它不是签名，不宣称防恶意本机管理员篡改。
3. 更新使用同目录私有临时文件、`write_all`、`sync_all`、原子替换和父目录/Windows write-through。
4. 原子替换前保留上一份已验证文档；启动时优先有效 primary，其次有效 backup，再创建默认值。
5. 未来只有安装会话 journal、并发状态或关系查询证明该方案不足，才能另行审查 SQLite；不得在执行时自行加入。

### 新增 crate 的逐项理由

| crate | 精确约束 | 允许用途 | 不使用更重方案的理由 |
| --- | --- | --- | --- |
| `serde_json = "1.0"` | production dependency | 权威 DTO、严格设置文档、JSONL、manifest | 已使用 serde；不引入数据库/配置框架 |
| `url = "2.5"` | default features | 代理 authority 解析、IDNA/IPv6/规范化 | 手写 URL parser 容易出现 userinfo、IPv6、percent encoding 差异 |
| `regex = "1.12"` | default features | 第二道文本脱敏 | 只使用编译时常量表达式；不接受动态 regex |
| `sha2 = "0.11"` | default features | 配置文档体与诊断条目 SHA-256 | 不引入签名、加密或通用 crypto framework |
| `zip = { version = "=8.6.0", default-features = false }` | `Stored` only | 标准 ZIP container | 不复制 V1 手写 ZIP/CRC/整数转换代码，不启用压缩、AES 或时间特性 |
| `libc = "0.2"` | macOS target only | `O_NOFOLLOW`/`O_CLOEXEC` | 防 TOCTOU 的 OS 原语，无 shell |
| `windows-sys = "0.61"` | Windows target only，最小 FileSystem/Security features | reparse/ACL/`ReplaceFileW`/`MoveFileExW` | 防 reparse 与权限泄露需要 OS 原语，无 COM/网络/提权 |

不得加入 `rusqlite`、`sqlite`、`chrono`、`time`、`tempfile`、`tokio`、`reqwest`、`ureq`、`hyper`、Tauri dialog/fs/http/opener/shell/process plugin 或前端包。依赖 API 依据：[`rusqlite::backup` 展示了 SQLite 需额外 backup feature 和双连接](https://docs.rs/rusqlite/latest/rusqlite/backup/)、[`url::Url` 提供 username/password/host/query/fragment 分解](https://docs.rs/url/latest/url/)、[`zip::write::SimpleFileOptions` 可固定权限](https://docs.rs/zip/latest/zip/write/type.SimpleFileOptions.html)、[`serde_json::Deserializer` 默认保留 recursion limit](https://docs.rs/serde_json/latest/serde_json/struct.Deserializer.html)。

### 固定 Downloads 与 native save dialog 对比

推荐固定 Downloads：

- command 只接受 `DiagnosticsExportTarget::Downloads`，没有任意路径、扩展名或 URL；
- 不增加 dialog plugin/capability，现有 CSP/capability 保持字节级不变；
- 文件名、冲突策略、权限和错误码可完全由 Rust 测试；
- 用户在 manifest 后点击“导出到下载文件夹”，仍是明确导出。

本切片不选 native save dialog，因为它需要新增 plugin、capability、前端路径/授权状态和更多平台 UI 验收，扩大攻击面且不是诊断功能的必要条件。后续若产品需要“另存为”，必须以新的 capability 审查替换固定 Downloads，不得在本计划执行中顺手加入。

### 反向审计必须守住的结论

- “配置了代理”不等于“测试代理”：Slice 2 无网络客户端，UI 必须明确只保存设置。
- `npmmirror` allowlist 不等于下载可信：实际回退必须在 Slice 9 结合官方校验值。
- 正则不是第一道防线：日志 API 不接受任意 map/环境/输出；正则和 URL/path scrub 是第二道防线。
- 配置 checksum 不是认证：它只用于损坏检测，不用于抵御有本机写权限的攻击者。
- 两文件无法跨平台同时原子更新：写入顺序必须保证任一崩溃点至少保留一个有效 primary/backup，恢复测试逐点验证。
- Downloads 可能是 symlink、reparse、只读、已满或不存在；这些都是稳定错误，不得退回桌面、临时目录或 app data 后谎称成功。
- V1 `custom` registry 即使看起来像 URL 也不能自由导入；只有规范化后精确等于两个内置 allowlist URL才可预览为候选。
- macOS 没有批准的 V1 路径，必须显示“此平台没有旧版配置来源”，不能扫描 `~/Library` 猜测。

---

## 权威 DTO 与接口草案

以下名称在所有任务中固定；实现者不得自行改名：

```rust
pub enum LanguageId { ZhCn }
pub enum NetworkMode { Direct, SystemProxy, ManualProxy }
pub enum NpmRegistryPolicy { OfficialOnly, OfficialThenNpmmirror }

pub struct NetworkSettings {
    pub mode: NetworkMode,
    pub manual_proxy_url: Option<String>,
    pub npm_registry_policy: NpmRegistryPolicy,
}

pub struct SettingsSnapshot {
    pub schema_version: u16, // 1
    pub revision: u64,
    pub language: LanguageId,
    pub network: NetworkSettings,
    pub onboarding_completed: bool,
}

pub struct UpdateSettingsRequest {
    pub expected_revision: u64,
    pub language: LanguageId,
    pub network: NetworkSettings,
    pub onboarding_completed: bool,
}

pub enum ConfigHealthStatus {
    Healthy,
    Migrated,
    RecoveredBackup,
    RecoveredDefaults,
    ReadOnlyFutureVersion,
    ReadOnlyUnsafePath,
}

pub struct ConfigHealth {
    pub status: ConfigHealthStatus,
    pub schema_version: u16,
    pub backup_available: bool,
}

pub enum LoggingHealthStatus { Healthy, ReadOnlyUnsafePath, Unavailable }
pub struct PrivacySnapshot {
    pub telemetry_enabled: bool, // always false
    pub log_retention_days: u16, // 30
    pub log_max_bytes: u64,      // 104_857_600
    pub logging_status: LoggingHealthStatus,
}

pub enum LegacyDiscoveryStatus {
    Available,
    NotFound,
    NoSourceForPlatform,
    UnsafePath,
    Unreadable,
}

pub struct LegacyConfigDiscovery {
    pub status: LegacyDiscoveryStatus,
    pub display_source: Option<String>,
}

pub enum LegacyPreviewField { NetworkMode, ManualProxy, NpmRegistry }
pub enum LegacyFieldDisposition {
    Eligible,
    IgnoredUnsupported,
    RejectedSecret,
    RejectedInvalid,
}
pub enum LegacyIgnoredCategory {
    CcSwitch,
    SubscriptionPage,
    OpenCode,
    Python,
    CustomDownloadSource,
    Unknown,
}
pub struct LegacyFieldPreview {
    pub field: LegacyPreviewField,
    pub disposition: LegacyFieldDisposition,
    pub display_value: Option<String>,
    pub selected_by_default: bool,
}
pub struct LegacyConfigPreview {
    pub schema_version: u16, // 1
    pub fields: Vec<LegacyFieldPreview>,
    pub ignored_categories: Vec<LegacyIgnoredCategory>,
    pub unknown_field_count: u32,
    pub secret_value_count: u32,
}

pub enum DiagnosticsExportTarget { Downloads }
pub struct DiagnosticsManifestEntry {
    pub archive_path: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
    pub redaction_policy_version: u16,
}
pub struct DiagnosticsManifest {
    pub schema_version: u16, // 1
    pub created_at_epoch_ms: u64,
    pub app_version: String,
    pub entries: Vec<DiagnosticsManifestEntry>,
    pub total_payload_bytes: u64,
}
pub struct DiagnosticsPlan {
    pub id: String,
    pub manifest_sha256: String,
    pub expires_at_epoch_ms: u64,
    pub manifest: DiagnosticsManifest,
}
pub struct ExportDiagnosticsRequest {
    pub plan_id: String,
    pub expected_manifest_sha256: String,
    pub target: DiagnosticsExportTarget,
}
pub struct DiagnosticsExportResult {
    pub file_name: String,
    pub display_location: String, // “下载/<leaf>”，永不返回 absolute path
    pub byte_length: u64,
    pub sha256: String,
}
```

`AppSnapshot` 固定增加：

```rust
pub struct AppSnapshot {
    pub schema_version: u16, // 3
    pub snapshot_version: u64,
    pub last_event_sequence: u64,
    pub platform: PlatformId,
    pub tools: Vec<ToolDefinition>,
    pub observations: Vec<ToolObservation>,
    pub detection_run: Option<DetectionRun>,
    pub settings: SettingsSnapshot,
    pub config_health: ConfigHealth,
    pub privacy: PrivacySnapshot,
    pub legacy_config: LegacyConfigDiscovery,
}
```

事件保持两个稳定 channel：

```rust
pub enum SettingsEventType {
    #[serde(rename = "settings.changed")]
    SettingsChanged,
}
pub struct SettingsEventEnvelope {
    pub schema_version: u16,
    pub sequence: u64,
    pub snapshot_version: u64,
    pub emitted_at_epoch_ms: u64,
    pub event_type: SettingsEventType,
    pub settings_revision: u64,
}
```

`CommandErrorCode` 增加：

```text
invalidSettings
revisionConflict
configReadOnly
unsafePath
diagnosticsUnavailable
diagnosticsPlanExpired
diagnosticsPlanChanged
exportNameConflict
exportDiskFull
legacyNoSource
legacyInvalid
legacyTooLarge
```

`CommandError` 增加可选生成字段 `field: Option<SettingsField>`；不得携带原始 error、path、URL、JSON 或 secret。

---

## 文件结构映射

### Rust 新增

- `apps/desktop/src-tauri/src/domain/network.rs`：代理 URL 与 registry allowlist 的纯规则。
- `apps/desktop/src-tauri/src/domain/config.rs`：设置文档 schema、checksum、迁移纯函数。
- `apps/desktop/src-tauri/src/domain/logging.rs`：类型化日志事件、级别、结构化 record。
- `apps/desktop/src-tauri/src/domain/diagnostics.rs`：safe archive path、manifest 不变量。
- `apps/desktop/src-tauri/src/domain/legacy.rs`：V1 严格 parser 与 preview 映射。
- `apps/desktop/src-tauri/src/platform/storage.rs`：安全路径/文件 trait 与固定 `AppPaths`。
- `apps/desktop/src-tauri/src/platform/fake_storage.rs`：Windows/macOS 语义可配置 fake。
- `apps/desktop/src-tauri/src/platform/native_storage.rs`：app data/Downloads/V1 固定路径及 OS 安全原语。
- `apps/desktop/src-tauri/src/infrastructure/mod.rs`
- `apps/desktop/src-tauri/src/infrastructure/config_store.rs`：原子 JSON、备份、隔离、恢复。
- `apps/desktop/src-tauri/src/infrastructure/redaction.rs`：文本/JSON/path 二次脱敏。
- `apps/desktop/src-tauri/src/infrastructure/log_store.rs`：JSONL 写入、轮换、保留。
- `apps/desktop/src-tauri/src/infrastructure/diagnostics_zip.rs`：暂存 ZIP 与固定 Downloads 导出。
- `apps/desktop/src-tauri/src/application/settings.rs`：revision 更新、snapshot mutation、事件。
- `apps/desktop/src-tauri/src/application/diagnostics.rs`：prepare/export 两阶段用例。
- `apps/desktop/src-tauri/src/application/legacy.rs`：discover/preview 只读用例。
- `apps/desktop/src-tauri/tests/config_recovery.rs`
- `apps/desktop/src-tauri/tests/privacy_diagnostics.rs`
- `apps/desktop/src-tauri/tests/slice_two_flow.rs`

### Rust 修改

- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/src/domain/contracts.rs`
- `apps/desktop/src-tauri/src/domain/mod.rs`
- `apps/desktop/src-tauri/src/platform/mod.rs`
- `apps/desktop/src-tauri/src/application/mod.rs`
- `apps/desktop/src-tauri/src/application/snapshot_store.rs`
- `apps/desktop/src-tauri/src/application/bootstrap.rs`
- `apps/desktop/src-tauri/src/api/commands.rs`
- `apps/desktop/src-tauri/src/api/events.rs`
- `apps/desktop/src-tauri/src/bin/export-bindings.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/tests/security_configuration.rs`
- `apps/desktop/src-tauri/tests/detection_flow.rs`

### React 新增

- `apps/desktop/src/features/settings/SettingsPanel.tsx`
- `apps/desktop/src/features/settings/SettingsPanel.test.tsx`
- `apps/desktop/src/features/diagnostics/DiagnosticsPanel.tsx`
- `apps/desktop/src/features/diagnostics/DiagnosticsPanel.test.tsx`
- `apps/desktop/src/features/legacy/LegacyConfigPreview.tsx`
- `apps/desktop/src/features/legacy/LegacyConfigPreview.test.tsx`

### React 修改

- `apps/desktop/src/shared/api/client.ts`
- `apps/desktop/src/shared/api/client.test.ts`
- `apps/desktop/src/features/detection/useDetectionSnapshot.ts`
- `apps/desktop/src/features/detection/useDetectionSnapshot.test.tsx`
- `apps/desktop/src/features/onboarding/Onboarding.tsx`
- `apps/desktop/src/features/onboarding/Onboarding.test.tsx`
- `apps/desktop/src/features/dashboard/StatusCenter.tsx`
- `apps/desktop/src/features/dashboard/StatusCenter.test.tsx`
- `apps/desktop/src/shared/i18n/zh-CN.ts`
- `apps/desktop/src/shared/i18n/zh-CN.test.ts`
- `apps/desktop/src/app/App.tsx`
- `apps/desktop/src/app/App.test.tsx`
- `apps/desktop/src/app/styles.css`
- `apps/desktop/src/shared/api/generated/*.ts`（只允许 exporter 生成）

### 文档修改

- `docs/testing/v2-platform-matrix.md`
- `README.md`

---

## 任务依赖图与提交切点

```text
Task 1 DTO/deps
  ├─ Task 2 network rules
  ├─ Task 3 secure storage boundary
  │    ├─ Task 4 atomic config
  │    ├─ Task 7 rotating logs
  │    ├─ Task 8 diagnostics zip
  │    └─ Task 9 V1 source
  ├─ Task 5 settings/snapshot/events (depends 2,4)
  ├─ Task 6 redaction
  │    ├─ Task 7
  │    └─ Task 8
  ├─ Task 10 IPC graph (depends 5,7,8,9)
  ├─ Task 11 frontend snapshot/API (depends 10)
  │    ├─ Task 12 settings UX
  │    └─ Task 13 diagnostics/V1 UX
  └─ Task 14 integration/security/docs (depends all)
```

每个 Task 的最后一步是独立提交；不得把多个任务压成一个巨型提交。Task 14 收口提交后工作树必须干净。

---

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

- HEAD 为 `8ada8ca` 或只包含已批准计划文档的直接后继。
- detached/隔离 worktree 可接受；不得切换或修改 `main`。
- 工作树起始干净。
- Node 主版本至少 24，npm 主版本至少 11，Rust 至少 1.88。
- Slice 1 typecheck、lint、Vitest、fmt、clippy、Rust unit/integration、binding drift 全部 PASS。

若失败，保存命令与首个失败测试名并停止；不得把既有红灯与 Slice 2 RED 混在一起，不得顺手升级工具链。

---

### Task 1：先固定依赖预算、Slice 2 DTO 与生成契约

**Files:**
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/src/domain/contracts.rs`
- Modify: `apps/desktop/src-tauri/src/bin/export-bindings.rs`
- Generate: `apps/desktop/src/shared/api/generated/*.ts`
- Test: `apps/desktop/src-tauri/src/domain/contracts.rs`
- Test: `apps/desktop/src-tauri/src/bin/export-bindings.rs`

**Interfaces:**
- Consumes: Slice 1 的 `AppSnapshot`、`CommandError`、`DetectionEventEnvelope`。
- Produces: 本文“权威 DTO 与接口草案”全部 DTO，供 Tasks 2–13 使用。

- [ ] **Step 1 (2–5 min, RED): 添加 snapshot/设置序列化测试**

在 `contracts.rs` 增加 `slice_two_snapshot_serializes_versioned_privacy_state`，构造最小 `AppSnapshot`，精确断言：

```rust
assert_eq!(json["schemaVersion"], 3);
assert_eq!(json["settings"]["revision"], 4);
assert_eq!(json["settings"]["network"]["mode"], "manualProxy");
assert_eq!(json["settings"]["network"]["npmRegistryPolicy"], "officialOnly");
assert_eq!(json["privacy"]["telemetryEnabled"], false);
assert_eq!(json["legacyConfig"]["status"], "noSourceForPlatform");
```

- [ ] **Step 2 (2–5 min, RED): 运行 DTO 测试**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml slice_two_snapshot_serializes_versioned_privacy_state
```

Expected: compile FAIL，首个错误为缺少 `SettingsSnapshot` 或 `AppSnapshot.settings`。

- [ ] **Step 3 (2–5 min, GREEN): 添加精确依赖**

把 `serde_json` 从 dev-only 移到 production，并只加入：

```toml
serde_json = "1.0"
url = "2.5"
regex = "1.12"
sha2 = "0.11"
zip = { version = "=8.6.0", default-features = false }

[target.'cfg(target_os = "macos")'.dependencies]
libc = "0.2"

[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.61", features = [
  "Win32_Foundation",
  "Win32_Security",
  "Win32_Security_Authorization",
  "Win32_Storage_FileSystem",
  "Win32_System_Memory",
] }
```

不得修改前端 `package.json`。

- [ ] **Step 4 (2–5 min, GREEN): 实现设置、健康与隐私 DTO**

按本文草案增加 `LanguageId`、`NetworkMode`、`NpmRegistryPolicy`、`NetworkSettings`、`SettingsSnapshot`、`UpdateSettingsRequest`、`SettingsField`、`ConfigHealth*`、`LoggingHealthStatus`、`PrivacySnapshot`；全部继续派生现有 serde/TS traits、camelCase、独立 `export_to`。

- [ ] **Step 5 (2–5 min, GREEN): 实现 legacy 与 diagnostics DTO**

按本文草案增加全部 Legacy/Diagnostics DTO。所有 `u64` 字段加：

```rust
#[ts(type = "number")]
```

`DiagnosticsExportTarget` 只能有 `Downloads` 一个成员。

- [ ] **Step 6 (2–5 min, GREEN): 扩展 AppSnapshot 和错误码**

把 `AppSnapshot` 增加 `settings/config_health/privacy/legacy_config`，测试 fixture 的 schema version 改为 3。给 `CommandError` 增加：

```rust
pub field: Option<SettingsField>,
```

现有 detection 错误映射使用 `field: None`。

- [ ] **Step 7 (2–5 min, RED→GREEN): 固定 settings.changed event**

先写 `settings_event_uses_stable_name_and_metadata_only`，Expected compile FAIL；再实现 `SettingsEventType`/`SettingsEventEnvelope`，序列化必须得到 `"settings.changed"`，payload 不含完整 settings 或 proxy。

- [ ] **Step 8 (2–5 min, RED): 更新 exporter 精确集合测试**

把 exporter 测试改为比较完整有序类型名列表，并断言旧集合缺少任何新 DTO 时失败。Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --bin export-bindings export_bindings_have_exact_contract_type_set
```

Expected: FAIL，显示 expected/actual 类型集合差异。

- [ ] **Step 9 (2–5 min, GREEN): 更新 exporter 并生成**

逐项 `TS::export`，保持显式 `export type` index。Run:

```bash
npm run generate:bindings
npm run check:bindings
```

Expected: generated 文件与 exporter 集合一一对应，drift check 退出 0。

- [ ] **Step 10 (2–5 min, REFACTOR): 依赖与协议守卫**

在 exporter/security 测试断言：

```text
Cargo.toml contains no rusqlite/sqlite/tokio/reqwest/ureq/hyper
apps/desktop/package.json unchanged
DiagnosticsExportTarget.ts contains only "downloads"
UpdateSettingsRequest.ts comes from exporter
```

Run:

```bash
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --all
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml contracts
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --bin export-bindings
git diff --check
```

Expected: PASS，无格式差异。

- [ ] **Step 11 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/Cargo.lock apps/desktop/src-tauri/src/domain/contracts.rs apps/desktop/src-tauri/src/bin/export-bindings.rs apps/desktop/src/shared/api/generated
git commit -m "feat: define configuration and diagnostics contracts"
```

---

### Task 2：用纯函数锁定代理规范化和 registry allowlist

**Files:**
- Create: `apps/desktop/src-tauri/src/domain/network.rs`
- Modify: `apps/desktop/src-tauri/src/domain/mod.rs`
- Test: `apps/desktop/src-tauri/src/domain/network.rs`

**Interfaces:**
- Consumes: `NetworkMode`、`NpmRegistryPolicy`。
- Produces:

```rust
pub struct NormalizedProxyUrl(String);
pub enum ProxyValidationError {
    Empty, TooLong, Invalid, UnsupportedScheme, Credentials,
    PathNotAllowed, QueryNotAllowed, FragmentNotAllowed,
}
pub fn normalize_proxy_url(input: &str) -> Result<NormalizedProxyUrl, ProxyValidationError>;
pub fn validate_network_settings(input: NetworkSettings) -> Result<NetworkSettings, SettingsValidationError>;
pub fn registry_chain(policy: NpmRegistryPolicy) -> &'static [RegistryDefinition];
```

- [ ] **Step 1 (2–5 min, RED): 写规范化表测试**

```rust
#[test]
fn normalizes_http_proxy_authorities_without_credentials() {
    let cases = [
        (" HTTP://Proxy.Example:8080/ ", "http://proxy.example:8080"),
        ("https://EXAMPLE.com:443/", "https://example.com"),
        ("http://[::1]:7890/", "http://[::1]:7890"),
        ("http://127.0.0.1:7890", "http://127.0.0.1:7890"),
    ];
    for (input, expected) in cases {
        assert_eq!(normalize_proxy_url(input).unwrap().as_str(), expected);
    }
}
```

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml normalizes_http_proxy_authorities_without_credentials
```

Expected: compile FAIL，`domain::network` 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 写最小规范化**

规则固定：

1. trim 仅允许首尾 Unicode whitespace；trim 后 1–2048 bytes。
2. `Url::parse` 后 scheme 只能 `http|https`。
3. raw authority 中出现 `@`、`url.username()` 非空或 `password()` 非空均拒绝。
4. host 必须存在；path 只能 `/`；query/fragment 必须 `None`。
5. host 采用 `url` 的小写/IDNA/IPv6 serialization；默认端口 80/443省略，其他端口保留。
6. 返回值去掉结尾 `/`，不得暴露可变 `Url`。

- [ ] **Step 4 (2–5 min, RED): 写恶意代理 corpus**

逐项断言稳定错误，不比较 parser 原始字符串：

```text
socks5://127.0.0.1:1080
socks://127.0.0.1:1080
ftp://proxy.example
http://user:pass@proxy.example
http://user%3Apass@proxy.example
http://@proxy.example
http:///missing-host
http://proxy.example/path
http://proxy.example?token=secret
http://proxy.example#fragment
http://proxy.example/\r\nInjected: yes
```

Run Expected: 至少 socks case FAIL，直到拒绝规则完整。

- [ ] **Step 5 (2–5 min, GREEN): 完成拒绝规则**

实现 `ProxyValidationError` 精确映射；不得把输入写入 `Display`/`Debug` error 文本。`NormalizedProxyUrl` 的 `Debug` 固定输出 `NormalizedProxyUrl([REDACTED])`。

- [ ] **Step 6 (2–5 min, RED→GREEN): 锁定设置组合**

先测试：

```rust
assert!(validate_network_settings(manual(None)).is_err());
assert_eq!(validate_network_settings(direct(Some("http://x"))).unwrap().manual_proxy_url, None);
assert_eq!(validate_network_settings(system(Some("http://x"))).unwrap().manual_proxy_url, None);
```

`manualProxy` 必须有有效 URL；其他模式忽略/清空 UI 残留 URL，避免休眠 secret-like 数据。

- [ ] **Step 7 (2–5 min, RED): 写 registry 精确矩阵**

```rust
assert_eq!(registry_chain(OfficialOnly), [official()]);
assert_eq!(registry_chain(OfficialThenNpmmirror), [official(), npmmirror()]);
```

并断言每项 HTTPS、无 userinfo/query/fragment，host/path 精确为：

```text
registry.npmjs.org /
registry.npmmirror.com /
```

- [ ] **Step 8 (2–5 min, GREEN): 实现编译时 allowlist**

`RegistryDefinition` 只含 `id: RegistryId` 和私有静态 URL；没有 `Custom` enum、没有 String constructor、没有 serde 输入类型。

- [ ] **Step 9 (2–5 min, REFACTOR): 源码安全守卫**

测试读取 production `network.rs` 并断言：

- 只出现两个 `https://` literal；
- 不出现 `socks`、`customRegistry`、`reqwest`、`ureq`、`connect(`；
- error formatting 不包含 `{input}`。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain::network
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS。

- [ ] **Step 10 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/domain
git commit -m "feat: validate offline network settings"
```

---

### Task 3：建立固定路径与无跟随安全文件边界

**Files:**
- Create: `apps/desktop/src-tauri/src/platform/storage.rs`
- Create: `apps/desktop/src-tauri/src/platform/fake_storage.rs`
- Create: `apps/desktop/src-tauri/src/platform/native_storage.rs`
- Modify: `apps/desktop/src-tauri/src/platform/mod.rs`
- Test: same files

**Interfaces:**
- Consumes: Tauri setup 提供的 app data/Downloads 基目录，Windows 读取固定 `%APPDATA%` 仅用于 V1。
- Produces:

```rust
pub struct AppPaths {
    pub state_dir: SecureDirectory,
    pub logs_dir: SecureDirectory,
    pub diagnostics_staging_dir: SecureDirectory,
    pub downloads_dir: Option<SecureDirectory>,
    pub legacy_config: Option<SecureFilePath>,
}

pub enum FixedLeaf {
    SettingsPrimary,
    SettingsBackup,
}

pub trait SecureStorage: Send + Sync {
    fn platform(&self) -> PlatformId;
    fn paths(&self) -> Result<AppPaths, StorageError>;
    fn read_regular_nofollow(&self, path: &SecureFilePath, max: u64) -> Result<Vec<u8>, StorageError>;
    fn create_private_new(&self, path: &SecureFilePath) -> Result<Box<dyn DurableWrite>, StorageError>;
    fn atomic_replace(&self, temp: &SecureFilePath, target: &SecureFilePath, backup: Option<&SecureFilePath>) -> Result<(), StorageError>;
    fn list_regular_nofollow(&self, dir: &SecureDirectory) -> Result<Vec<SecureFilePath>, StorageError>;
    fn remove_regular_nofollow(&self, path: &SecureFilePath) -> Result<(), StorageError>;
    fn free_space_bytes(&self, dir: &SecureDirectory) -> Result<u64, StorageError>;
}
```

`SecureFilePath` 只能由 fixed leaf 或 `SafeGeneratedLeaf::parse` 创建；不实现 public `From<String>`。

- [ ] **Step 1 (2–5 min, RED): 写 fake 双平台路径测试**

测试 Windows fake 得到：

```text
<appData>/state/settings.json
<appData>/state/settings.backup.json
<appData>/logs
<appData>/diagnostics-staging
<downloads>
%APPDATA%/ai-coding-installer/config.json
```

macOS fake 的 `legacy_config` 必须 `None`。

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml fake_storage_builds_only_fixed_platform_paths
```

Expected: compile FAIL，storage trait 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现路径 newtype**

`SafeGeneratedLeaf::parse` 固定拒绝：

```text
empty, ".", "..", slash, backslash, NUL, colon,
absolute/root/prefix components, >120 ASCII bytes,
non [a-zA-Z0-9._-]
```

固定 leaf 不经过用户字符串。display API 只能返回 `%APPDATA%/...`、`~/...`、`下载/<leaf>` 等占位路径。

- [ ] **Step 4 (2–5 min, RED→GREEN): fake 记录无跟随/私有标志**

Fake 每次 open 记录：

```rust
StorageOpenRecord {
    operation: StorageOperation,
    no_follow: true,
    private_permissions: true,
    path_role: StoragePathRole,
}
```

测试 config/log/diagnostic/legacy open 全部 `no_follow=true`；legacy 永远 read-only。

- [ ] **Step 5 (2–5 min, RED): 写 symlink/reparse 与 TOCTOU contract**

Fake 可在 `before_open` 注入 `SwapToLink`；断言 read/create/replace 返回 `StorageError::UnsafeLink`，没有 fallback open。再断言 target 已存在 symlink、hard link count >1、目录中间组件为 app-owned symlink 时拒绝。

- [ ] **Step 6 (2–5 min, GREEN): 实现 macOS native 边界**

仅在 `native_storage.rs` 的 macOS cfg 使用：

- app-owned 目录创建后 mode `0700`；
- 文件 mode `0600`；
- `open` 使用 `O_NOFOLLOW | O_CLOEXEC`；
- 打开后 `fstat` 再确认 regular file、link count=1、uid 为当前用户；
- temp 与 target 必须同目录；
- `rename` 后打开父目录并 `sync_all`；
- `ENOSPC` 映射 `DiskFull`，`ELOOP` 映射 `UnsafeLink`。

不调用 shell、`chmod` command 或 Authorization Services。

- [ ] **Step 7 (2–5 min, GREEN): 实现 Windows native 边界**

使用 `windows-sys`：

- app-owned 目录使用受保护 DACL，只允许 Owner/SYSTEM/Administrators full control；
- handle 使用 `FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_WRITE_THROUGH`；
- 打开后拒绝 `FILE_ATTRIBUTE_REPARSE_POINT` 和非 disk file；
- 替换已有 primary 用 `ReplaceFileW`，同时写固定 backup；首次创建用 `MoveFileExW(MOVEFILE_WRITE_THROUGH)`；
- `ERROR_DISK_FULL|ERROR_HANDLE_DISK_FULL` → `DiskFull`；
- `ERROR_ALREADY_EXISTS|ERROR_FILE_EXISTS` → `AlreadyExists`；
- 不使用 `ShellExecuteW`、`runas`、COM dialog 或 token elevation。

- [ ] **Step 8 (2–5 min, RED→GREEN): native 权限静态/集成测试**

macOS test 验证 mode；Windows test 读取 DACL 并断言不存在 Everyone/Authenticated Users/Users 的 allow ACE。两端都创建 symlink/reparse fixture（权限不允许时明确 SKIP 文本只限该 native fixture，fake contract 必须 PASS）。

- [ ] **Step 9 (2–5 min, REFACTOR): 目录边界回归**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml platform::storage
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml platform::fake_storage
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml platform::native_storage
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS；domain/application 仍不直接使用 `std::fs/std::env`。

- [ ] **Step 10 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/platform apps/desktop/src-tauri/Cargo.toml apps/desktop/src-tauri/Cargo.lock
git commit -m "security: add private fixed-path storage"
```

---

### Task 4：实现版本化原子设置、迁移、备份与损坏恢复

**Files:**
- Create: `apps/desktop/src-tauri/src/domain/config.rs`
- Create: `apps/desktop/src-tauri/src/infrastructure/mod.rs`
- Create: `apps/desktop/src-tauri/src/infrastructure/config_store.rs`
- Create: `apps/desktop/src-tauri/tests/config_recovery.rs`
- Modify: `apps/desktop/src-tauri/src/domain/mod.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Test: same files

**Interfaces:**
- Consumes: Task 2 validation、Task 3 `SecureStorage`、`Clock`。
- Produces:

```rust
pub const CONFIG_SCHEMA_VERSION: u16 = 1;
pub struct ConfigPayload {
    pub language: LanguageId,
    pub network: NetworkSettings,
    pub onboarding_completed: bool,
}
pub struct ConfigBody {
    pub format: String,
    pub schema_version: u16,
    pub migrated_from_schema_version: Option<u16>,
    pub revision: u64,
    pub payload: ConfigPayload,
}
pub struct ConfigDocument {
    #[serde(flatten)]
    pub body: ConfigBody,
    pub document_sha256: String,
}
pub struct ConfigStartup {
    pub settings: SettingsSnapshot,
    pub health: ConfigHealth,
}
pub trait ConfigRepository: Send + Sync {
    fn load(&self) -> Result<ConfigStartup, ConfigStoreError>;
    fn replace(&self, request: &UpdateSettingsRequest) -> Result<ConfigStartup, ConfigStoreError>;
}
```

- [ ] **Step 1 (2–5 min, RED): 写默认文档与 checksum 测试**

```rust
#[test]
fn default_config_is_strict_version_one_with_document_checksum() {
    let doc = ConfigDocument::defaults();
    assert_eq!(doc.body.schema_version, 1);
    assert_eq!(doc.body.migrated_from_schema_version, None);
    assert_eq!(doc.body.revision, 1);
    assert_eq!(doc.body.payload.network.mode, NetworkMode::Direct);
    assert_eq!(doc.body.payload.network.npm_registry_policy, NpmRegistryPolicy::OfficialOnly);
    assert!(doc.verify_checksum());
}
```

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml default_config_is_strict_version_one_with_document_checksum
```

Expected: compile FAIL，config domain 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现 canonical document checksum**

只对不含 checksum 的固定字段 struct `ConfigBody` 的 `serde_json::to_vec` 计算 SHA-256 lowercase hex，覆盖 format、schema、migration source、revision 和 payload。反序列化 DTO 使用 `#[serde(deny_unknown_fields)]`；format、hash 长度/字符、schema、revision 全部严格校验。不得 hash `HashMap`。

- [ ] **Step 4 (2–5 min, RED): 写 V0→V1 迁移 fixture**

V0 精确格式：

```json
{
  "schemaVersion": 0,
  "revision": 4,
  "language": "zhCn",
  "networkMode": "manualProxy",
  "manualProxyUrl": "HTTP://127.0.0.1:7890/",
  "npmRegistry": "npmmirror",
  "onboardingCompleted": true
}
```

断言迁移到 revision 5、`migratedFromSchemaVersion=0`、normalized proxy、`officialThenNpmmirror`、有效 checksum；第二次调用 `migrate_document` 返回 `AlreadyCurrent` 且 bytes 不变。

- [ ] **Step 5 (2–5 min, GREEN): 实现唯一迁移链**

`parse_and_migrate(bytes)` 先用只含 `schemaVersion` 的 probe 读取版本：

- 0 → 严格 `ConfigDocumentV0` → validate/normalize → V1；
- 1 → 严格 `ConfigDocument`；
- >1 → `FutureVersion`，不得覆盖；
- 缺失/负数/非整数/unknown field/checksum mismatch → `Corrupt`。

保留 serde_json 默认 recursion limit，不启用 `unbounded_depth`。

- [ ] **Step 6 (2–5 min, RED): 写原子写入 failpoint 矩阵**

FakeStorage failpoint：

```text
afterTempCreate
afterTempWrite
afterTempSync
afterBackupReplace
afterPrimaryReplace
beforeParentSync
```

每点模拟崩溃后重新 `load()`，Expected：只能得到完整旧值或完整新值；永不接受半 JSON，至少 primary/backup 一份有效。

- [ ] **Step 7 (2–5 min, GREEN): 实现原子 replace**

固定顺序：

1. repository `Mutex` 锁住 load/replace。
2. 读取并验证当前 primary。
3. 验证 request revision 与完整 network settings。
4. checked increment revision，生成新 bytes。
5. 在同一 state dir 以 `settings.tmp-<pid>-<counter>.json` `create_new`。
6. `write_all`、`sync_all`。
7. 若平台 `atomic_replace` 支持 backup 参数，一次替换；否则先把当前有效 bytes 原子写到 backup，再替换 primary。
8. 同步目录；仅删除本次 temp。
9. 重读 primary 校验后返回。

任一步失败不写默认值掩盖错误。

- [ ] **Step 8 (2–5 min, RED): 写损坏恢复矩阵**

`config_recovery.rs` 覆盖：

| primary | backup | Expected |
| --- | --- | --- |
| valid V1 | any | primary + healthy |
| valid V0 | absent | migrated + backup of V0 |
| corrupt/truncated | valid V1 | quarantine primary + recoveredBackup |
| checksum mismatch | valid V1 | recoveredBackup |
| corrupt | corrupt/absent | quarantine evidence + recoveredDefaults |
| future version | valid older backup | readOnlyFutureVersion，不回退旧值 |
| symlink/reparse | valid backup | readOnlyUnsafePath，不跟随/覆盖 |

- [ ] **Step 9 (2–5 min, GREEN): 实现恢复**

quarantine leaf 固定 `settings.corrupt-<epoch_ms>-<counter>.json`，仍经 `SafeGeneratedLeaf`；不把原始内容放日志/DTO。Future/unsafe path 只读启动，`replace` 返回稳定错误。默认恢复必须写新 primary 和 backup，再返回 `RecoveredDefaults`。

- [ ] **Step 10 (2–5 min, RED→GREEN): 并发 revision 测试**

两个线程同用 revision 4 更新；恰好一个成功为 revision 5，另一个 `RevisionConflict`。重开 repository 仍为成功值。不得 last-write-wins。

- [ ] **Step 11 (2–5 min, REFACTOR): 恶意文档 corpus**

覆盖 unknown field、duplicate top-level key、10,000 层嵌套、10 MiB file、NaN、负 revision、超长 URL、custom registry、secret proxy。Duplicate key 必须用自定义 serde `MapAccess` visitor 拒绝，不能接受 last-wins。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain::config
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test config_recovery
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS。

- [ ] **Step 12 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/domain apps/desktop/src-tauri/src/infrastructure apps/desktop/src-tauri/tests/config_recovery.rs apps/desktop/src-tauri/src/lib.rs
git commit -m "feat: persist atomic versioned settings"
```

---

### Task 5：把 durable settings 接入 authoritative snapshot 与 settings.changed

**Files:**
- Create: `apps/desktop/src-tauri/src/application/settings.rs`
- Modify: `apps/desktop/src-tauri/src/application/mod.rs`
- Modify: `apps/desktop/src-tauri/src/application/snapshot_store.rs`
- Modify: `apps/desktop/src-tauri/src/application/bootstrap.rs`
- Modify: `apps/desktop/src-tauri/src/api/events.rs`
- Test: same files

**Interfaces:**
- Consumes: `ConfigRepository`、`SnapshotStore`、Task 1 event DTO。
- Produces:

```rust
pub trait SettingsEventSink: Send + Sync {
    fn publish(&self, event: SettingsEventEnvelope) -> Result<(), EventPublishError>;
}
pub struct SettingsService {
    repository: Arc<dyn ConfigRepository>,
    snapshots: Arc<SnapshotStore>,
    events: Arc<dyn SettingsEventSink>,
}
impl SettingsService {
    pub fn update(&self, request: UpdateSettingsRequest) -> Result<SettingsSnapshot, UpdateSettingsError>;
}
pub fn SnapshotStore::replace_settings(
    &self,
    settings: SettingsSnapshot,
    health: ConfigHealth,
) -> Result<SettingsEventEnvelope, SnapshotMutationError>;
```

- [ ] **Step 1 (2–5 min, RED): 更新 SnapshotStore 初始化测试**

`SnapshotStore::new` 增加 settings/health/privacy/legacy 参数；初始 snapshot schema=3。Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml snapshot_store_versions_each_mutation_and_never_changes_on_read
```

Expected: compile FAIL，constructor/fields 不匹配。

- [ ] **Step 2 (2–5 min, GREEN): 扩展 store 初始化**

保持 Slice 1 不变量：初始 snapshotVersion=1、sequence=0；读取不递增。检测 mutation 不改 settings；settings mutation 不改 observations/run。

- [ ] **Step 3 (2–5 min, RED): 写 settings mutation 测试**

断言 revision 2 替换：

```text
snapshotVersion 1→2
lastEventSequence 0→1
event.sequence=1
event.settingsRevision=2
event.eventType=settings.changed
detection facts byte-for-byte unchanged
```

- [ ] **Step 4 (2–5 min, GREEN): 实现原子内存 mutation**

在同一 store mutex 内 checked increment，替换 settings/health，生成 metadata-only event。拒绝低于或等于当前 revision，错误 `StaleSettingsRevision`。

- [ ] **Step 5 (2–5 min, RED): 写 service 提交顺序测试**

Fake repository 和 sink 记录顺序：

```text
repo.replace → store.replace_settings → sink.publish
```

repository 失败时 snapshot/event 均不变；event sink 失败时 durable config 与 snapshot 保持已提交。

- [ ] **Step 6 (2–5 min, GREEN): 实现 SettingsService**

错误映射：

| internal | `CommandErrorCode` | retryable | field |
| --- | --- | ---: | --- |
| proxy invalid | invalidSettings | false | manualProxyUrl |
| revision mismatch | revisionConflict | true | none |
| future/unsafe read-only | configReadOnly/unsafePath | false | none |
| IO transient | internal | true | none |

不得把 request 或 repository error 字符串放错误 DTO。

- [ ] **Step 7 (2–5 min, RED→GREEN): 事件失败恢复**

测试 sink 永远失败后：

1. `update()` 返回成功 settings；
2. bootstrap 返回新 revision；
3. `snapshotVersion/sequence` 已递增；
4. 前端未来 focus/bootstrap 可恢复。

- [ ] **Step 8 (2–5 min, RED→GREEN): 持久化 onboarding**

测试把 `onboarding_completed` false→true 写入，重建 repository/store 后 bootstrap 仍 true；不使用 localStorage。

- [ ] **Step 9 (2–5 min, REFACTOR): 并发读写不变量**

8 reader + 2 conflicting writers，所有 snapshot 满足：

```text
snapshotVersion == lastEventSequence + 1
settings.revision 单调
network manual mode iff proxy URL present
facts/run 不出现半写
```

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml application::settings
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml snapshot_store
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS。

- [ ] **Step 10 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/application apps/desktop/src-tauri/src/api/events.rs
git commit -m "feat: publish durable settings snapshots"
```

---

### Task 6：先以字段最小化，再用双层算法脱敏

**Files:**
- Create: `apps/desktop/src-tauri/src/domain/logging.rs`
- Create: `apps/desktop/src-tauri/src/infrastructure/redaction.rs`
- Modify: `apps/desktop/src-tauri/src/domain/mod.rs`
- Modify: `apps/desktop/src-tauri/src/infrastructure/mod.rs`
- Test: same files

**Interfaces:**
- Consumes: `PlatformId`、稳定业务 error/status code、home roots（只保存在 redactor 内存）。
- Produces:

```rust
pub const REDACTION_POLICY_VERSION: u16 = 1;
pub enum LogLevel { Info, Warn, Error }
pub enum LogEvent {
    AppStarted { platform: PlatformId },
    ConfigStartup { outcome: ConfigHealthStatus, schema_version: u16 },
    SettingsUpdated { revision: u64, mode: NetworkMode, registry: NpmRegistryPolicy },
    DiagnosticsPrepared { entry_count: u32, total_bytes: u64 },
    DiagnosticsExported { entry_count: u32, total_bytes: u64 },
    LegacyPreviewed { outcome: LegacyDiscoveryStatus, eligible: u32, ignored: u32, secret_values: u32 },
    DetectionRunChanged { run_id: String, status: DetectionRunStatus },
    LogRecordDiscarded { reason: LogDiscardReason },
}
pub struct LogRecord {
    pub schema_version: u16,
    pub timestamp_epoch_ms: u64,
    pub level: LogLevel,
    pub event: LogEvent,
}
pub struct PrivatePathRoot(String);
pub struct Redactor {
    private_roots: Vec<PrivatePathRoot>,
}
impl Redactor {
    pub fn redact_text(&self, input: &str) -> String;
    pub fn redact_json_value(&self, value: &mut serde_json::Value);
    pub fn sanitize_log_record(&self, record: &LogRecord) -> Result<Vec<u8>, RedactionError>;
}
```

没有 `LogEvent::Message(String)`，没有 arbitrary map，没有环境、command、stdout/stderr 字段。

- [ ] **Step 1 (2–5 min, RED): 写类型化事件 schema 测试**

序列化 `SettingsUpdated`，断言只含 revision/mode/registry；production enum 源码不含 `proxy_url`、`environment`、`stdout`、`stderr`、`command_line`、`cookie`。

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml log_event_schema_has_no_arbitrary_or_secret_fields
```

Expected: compile FAIL，logging domain 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现最小类型化事件**

全部 enum 使用稳定 camelCase/tagged serialization；`run_id` 只接受 `run-[0-9]+` validator，不接受用户字符串。`LogRecord` 构造器从 `Clock` 取时间。

- [ ] **Step 4 (2–5 min, RED): 写必须脱敏 corpus**

同一测试串包含并逐一断言 secret 不在输出：

```text
Authorization: Bearer bearer-secret
Proxy-Authorization: Basic base64secret
Cookie: session=secret
Set-Cookie: session=secret
OPENAI_API_KEY=sk-openai-secret
ANTHROPIC_API_KEY=sk-ant-secret
token=query-secret
api_key=another-secret
password=proxy-secret
http://user:pass@127.0.0.1:7890
https://example.test/path?token=secret&x=1#frag
C:\Users\Alice\secret.txt
/Users/alice/secret.txt
```

- [ ] **Step 5 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml redacts_required_secret_path_and_url_shapes
```

Expected: compile FAIL，Redactor 不存在。

- [ ] **Step 6 (2–5 min, GREEN): 实现有序脱敏 pipeline**

固定顺序：

1. 输入最多 32 KiB；更长先截断并加 `[TRUNCATED]`，不在多字节 UTF-8 中间切断。
2. 把 CR/LF/TAB 之外控制符替换为 `�`，CR/LF 变 `\\r`/`\\n` 防 JSONL injection。
3. 已知 home roots 同时按 native separator、另一 separator、Windows case-insensitive 形式替换为 `<HOME>`。
4. 通用 `/Users/<segment>`、`C:\Users\<segment>` 根替换为 `<HOME>`，不保留用户名。
5. URL match 用 `Url` 解析；userinfo 替换为 `[REDACTED]@`，query 全部变 `?<REDACTED_QUERY>`，fragment 删除。
6. header/key-value 常量 regex 替换为 `[REDACTED]`。
7. 常见 token shapes：`sk-`、`sk-ant-`、`gh[pousr]_`、`xox[baprs]-`、长 JWT 三段，替换为 `[REDACTED_TOKEN]`。

Regex 通过 `OnceLock` 编译，pattern 为源码常量；不接受外部 regex。

- [ ] **Step 7 (2–5 min, RED→GREEN): 递归 JSON 脱敏**

先测试嵌套 object/array 的所有 string 被 scrub；key 名若匹配 `authorization|cookie|token|secret|password|apiKey`，value 无论类型都替换 string `[REDACTED]`。实现显式 stack，最大 64 depth/4096 nodes，避免递归栈。

- [ ] **Step 8 (2–5 min, RED→GREEN): 双重脱敏幂等**

断言：

```rust
let once = redactor.redact_text(input);
let twice = redactor.redact_text(&once);
assert_eq!(once, twice);
assert!(!once.contains(secret));
```

对固定 10,000 个 LCG 生成的 ASCII/UTF-8 corpus 循环：不得 panic、输出 <= 32 KiB + marker、已注入 secret 不泄漏。无需新增 property-test crate。

- [ ] **Step 9 (2–5 min, REFACTOR): JSONL 有效性**

`sanitize_log_record` 必须先转 `serde_json::Value`、递归 scrub、再 `to_vec`，最后加单个 `\n`。测试每条都能独立 parse，且内部 CR/LF 不产生第二条。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain::logging
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml infrastructure::redaction
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS。

- [ ] **Step 10 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/domain/logging.rs apps/desktop/src-tauri/src/domain/mod.rs apps/desktop/src-tauri/src/infrastructure/redaction.rs apps/desktop/src-tauri/src/infrastructure/mod.rs
git commit -m "feat: redact structured local events"
```

---

### Task 7：实现并发/崩溃安全的日志写入、轮换与双边界保留

**Files:**
- Create: `apps/desktop/src-tauri/src/infrastructure/log_store.rs`
- Create: `apps/desktop/src-tauri/tests/privacy_diagnostics.rs`
- Modify: `apps/desktop/src-tauri/src/infrastructure/mod.rs`
- Test: same files

**Interfaces:**
- Consumes: `SecureStorage`、`Redactor`、`Clock`、`LogRecord`。
- Produces:

```rust
pub const LOG_RETENTION_DAYS: u16 = 30;
pub const LOG_TOTAL_MAX_BYTES: u64 = 100 * 1024 * 1024;
pub const LOG_FILE_MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const LOG_RECORD_MAX_BYTES: usize = 16 * 1024;

pub trait AuditLogger: Send + Sync {
    fn write(&self, level: LogLevel, event: LogEvent) -> Result<(), LogStoreError>;
    fn snapshot_for_export(&self) -> Result<Vec<SanitizedLogFile>, LogStoreError>;
}
struct LogWriterState {
    utc_day: u64,
    sequence: u16,
    bytes_written: u64,
    active: Option<Box<dyn DurableWrite>>,
}
pub struct LocalLogStore {
    storage: Arc<dyn SecureStorage>,
    redactor: Arc<Redactor>,
    clock: Arc<dyn Clock>,
    state: Mutex<LogWriterState>,
}
```

- [ ] **Step 1 (2–5 min, RED): 写 UTC 日/大小边界测试**

Fake clock 在 day 100/101；写入刚好 5 MiB 与多 1 byte。Expected：

- `current_size + record_size <= 5 MiB` 不滚；
- `>5 MiB` 创建下一 sequence；
- UTC day 改变即新文件；
- 文件名 `code-ready-00000100-0000.jsonl`。

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml rotates_only_after_day_or_size_boundary
```

Expected: compile FAIL，LocalLogStore 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现单 writer 临界区**

每次 `write`：

1. 在进入 lock 前构造 typed record，不含任意 input。
2. lock 内再次 redact/serialize。
3. >16 KiB 返回 `RecordTooLarge`，不得写截断 JSON。
4. 清理 age/total。
5. 选择 active day/sequence，必要时 `create_private_new`。
6. 在同一 handle 单次 `write_all(record_line)`，然后 `sync_data`。
7. 再执行总量清理，确保返回时 <=100 MiB。

- [ ] **Step 4 (2–5 min, RED→GREEN): 30 天先到边界**

day 100 写文件，clock 到 day 130 仍保留（age=30）；day 131 删除。不得使用可伪造 mtime，age 从经过 parser 的固定 filename day 计算。

- [ ] **Step 5 (2–5 min, RED→GREEN): 100 MiB 先到边界**

构造多个 closed files；超出 1 byte 时按 `(day, sequence)` 最旧优先删，active 最后保留。若单个合法 active + record 无法满足总量，先 roll 再删旧 active；绝不删非匹配名称文件。

- [ ] **Step 6 (2–5 min, RED): 并发写测试**

16 threads × 200 records；导出 snapshot 后：

```text
3200 valid JSON lines
no interleaving/partial interior lines
monotonic per-file append order is not required across threads
total <=100MiB
```

- [ ] **Step 7 (2–5 min, GREEN): 完成 Mutex/handle 生命周期**

poisoned mutex 返回稳定 `Unavailable`，不 panic；禁止为性能删除 `sync_data`。LocalLogStore 不 Clone file handle 到锁外。

- [ ] **Step 8 (2–5 min, RED→GREEN): 崩溃尾行处理**

fixture 最后一条截断 JSON、内部一条损坏 JSON。`snapshot_for_export`：

- 对每条重新 parse + redactor；
- 丢弃最后不完整行并加入一条 typed `logRecordDiscarded/truncatedTail`；
- 内部损坏行替换为 `logRecordDiscarded/invalidJson`；
- 不把原字节复制进诊断。

- [ ] **Step 9 (2–5 min, RED→GREEN): symlink/权限/磁盘满**

FakeStorage 覆盖：

- active leaf 在 open 前换成 link → `UnsafePath`，零写入；
- existing log link/hardlink → 不读取/不删除；
- `sync_data` ENOSPC → `DiskFull`；
- cleanup delete 权限失败 → write 返回 `RetentionFailed`，不越过 100 MiB 继续写。

- [ ] **Step 10 (2–5 min, REFACTOR): 无秘密回归**

写入 Task 6 全 secret corpus，然后直接读取 FakeStorage bytes；每个 secret 均不存在。源码守卫断言 `LocalLogStore` 没有接收 `ProcessOutcome`/`std::env::vars`/arbitrary String message。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml infrastructure::log_store
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test privacy_diagnostics log_
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS。

- [ ] **Step 11 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/infrastructure/log_store.rs apps/desktop/src-tauri/src/infrastructure/mod.rs apps/desktop/src-tauri/tests/privacy_diagnostics.rs
git commit -m "feat: rotate private redacted logs"
```

---

### Task 8：生成可审计 manifest、安全 ZIP 与固定 Downloads 导出

**Files:**
- Create: `apps/desktop/src-tauri/src/domain/diagnostics.rs`
- Create: `apps/desktop/src-tauri/src/infrastructure/diagnostics_zip.rs`
- Create: `apps/desktop/src-tauri/src/application/diagnostics.rs`
- Modify: `apps/desktop/src-tauri/src/domain/mod.rs`
- Modify: `apps/desktop/src-tauri/src/infrastructure/mod.rs`
- Modify: `apps/desktop/src-tauri/src/application/mod.rs`
- Modify: `apps/desktop/src-tauri/tests/privacy_diagnostics.rs`
- Test: same files

**Interfaces:**
- Consumes: settings/health/privacy snapshot、`AuditLogger::snapshot_for_export`、`SecureStorage`、`Clock`。
- Produces:

```rust
pub struct DiagnosticsService {
    storage: Arc<dyn SecureStorage>,
    logger: Arc<dyn AuditLogger>,
    snapshots: Arc<SnapshotStore>,
    clock: Arc<dyn Clock>,
    active_plan: Mutex<Option<StagedDiagnostics>>,
}
impl DiagnosticsService {
    pub fn prepare(&self) -> Result<DiagnosticsPlan, DiagnosticsError>;
    pub fn export(&self, request: ExportDiagnosticsRequest) -> Result<DiagnosticsExportResult, DiagnosticsError>;
}
pub struct SafeArchivePath(String);
pub struct StagedDiagnostics {
    pub plan: DiagnosticsPlan,
    staging_file: SecureFilePath,
    archive_sha256: String,
    archive_bytes: u64,
}
```

- [ ] **Step 1 (2–5 min, RED): 写 safe archive path corpus**

接受：

```text
system.json
settings.json
logs/code-ready-00000100-0000.jsonl
```

拒绝：

```text
/absolute
C:\absolute
../escape
logs/../../escape
logs\evil
./dot
name\0x
empty segment
duplicate normalized name
```

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml safe_archive_path_rejects_zip_slip_names
```

Expected: compile FAIL，diagnostics domain 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现路径与 manifest 不变量**

`SafeArchivePath` 只允许 ASCII `[a-z0-9._/-]`、relative、无空/`.`/`..` segment、最多 160 bytes。Manifest entries 按 archive path 排序、无重复，最多 512 条，单项最多 16 MiB，总 payload 最多 110 MiB，checked arithmetic。

- [ ] **Step 4 (2–5 min, RED): 写诊断 payload 内容测试**

`prepare` 计划必须只含：

```text
system.json
settings.json
config-health.json
privacy.json
logs/*.jsonl
manifest.json (ZIP 内存在，但 manifest.entries 不自引用)
```

`settings.json` 只含：

```json
{
  "schemaVersion": 1,
  "revision": 4,
  "language": "zhCn",
  "networkMode": "manualProxy",
  "manualProxyConfigured": true,
  "npmRegistryPolicy": "officialOnly",
  "onboardingCompleted": true
}
```

不得含 manual proxy URL、V1 content、absolute path、hostname、username、env。

- [ ] **Step 5 (2–5 min, GREEN): 实现 prepare payload builder**

每个 payload 在内存中再次 `redact_json_value`/日志二次 scrub，计算 exact byte length/SHA-256。`manifest.json` 最后生成；manifest hash 返回 plan。plan id 为 `diag-<epoch_ms>-<atomic_counter>`，有效期固定 10 分钟。

- [ ] **Step 6 (2–5 min, RED): 写 ZIP 元数据/边界测试**

打开生成 ZIP 并断言：

- entry name 精确；
- compression `Stored`；
- unix mode `0600`；
- 无 directory/symlink entry；
- CRC/size 由 crate 验证；
- manifest 的每个 hash/size 与解包 bytes 一致；
- 恶意/重复 name 在写文件前被拒绝。

- [ ] **Step 7 (2–5 min, GREEN): 实现私有 staging**

在 app-owned `diagnostics-staging`：

1. 先清理本应用固定命名且过期的 regular staging；links 不碰。
2. `diagnostics-<plan>.zip.part` create_new/0600/nofollow。
3. `ZipWriter` + `SimpleFileOptions` + Stored + 0600。
4. finish、sync_all，原子 rename 为 `.zip`，sync dir。
5. `DiagnosticsPlan` 返回后暂存存在，但未复制到 Downloads。

- [ ] **Step 8 (2–5 min, RED): 写 export 名称冲突测试**

目标 leaf 固定：

```text
code-ready-diagnostics-<epoch_ms>-<plan_counter>.zip
```

若目标已存在 regular file、symlink 或 race 时 `create_new` 报 exists，返回 `ExportNameConflict`；不得覆盖、追加数字重试或删除用户文件。

- [ ] **Step 9 (2–5 min, GREEN): 实现固定 Downloads 复制**

`export` 验证 plan id、10 分钟 expiry、expected manifest hash；再：

1. Downloads 缺失 → `ExportUnavailable`；
2. free space 必须 >= staged bytes + 1 MiB safety margin；
3. Downloads/temp/target 全部通过 SecureStorage；
4. stream staged → Downloads `.part`，同时重新 SHA-256；
5. hash/size 必须等于 staged；
6. sync temp，atomic no-overwrite rename target，sync Downloads；
7. 成功才删除 staged；
8. 返回 safe leaf 与 `下载/<leaf>`，不返回 absolute path。

- [ ] **Step 10 (2–5 min, RED→GREEN): TOCTOU/权限/磁盘满矩阵**

在这些点注入：

```text
afterFreeSpaceCheck
afterTargetChecked
afterTargetTempCreate
midCopy
afterTargetSync
beforeFinalRename
```

Expected：

- symlink/reparse swap → unsafePath；
- midCopy ENOSPC → exportDiskFull，清理自己 `.part`，保留 staging；
- permission denied → diagnosticsUnavailable；
- crash 前 target 不存在，或存在完整 hash 匹配 target；永不声称半文件成功；
- 不自动改导出位置。

- [ ] **Step 11 (2–5 min, RED→GREEN): plan 过期/变化**

过期返回 `diagnosticsPlanExpired` 并安全删除 staging；错误 manifest hash 返回 `diagnosticsPlanChanged`，不导出。第二次 export 同 plan 返回 expired/unavailable，不覆盖第一次结果。

- [ ] **Step 12 (2–5 min, REFACTOR): 全包 secret scan**

把 Task 6 corpus 写入 logs/config fake values，prepare 后遍历 ZIP 每个 entry bytes；断言 secret、home root、proxy URL、query、V1 raw JSON 均不存在。包中 URL literal 只允许 registry policy ID，不允许 registry/proxy URL。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain::diagnostics
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml infrastructure::diagnostics_zip
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml application::diagnostics
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test privacy_diagnostics diagnostics_
```

Expected: PASS。

- [ ] **Step 13 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/domain/diagnostics.rs apps/desktop/src-tauri/src/domain/mod.rs apps/desktop/src-tauri/src/infrastructure apps/desktop/src-tauri/src/application apps/desktop/src-tauri/tests/privacy_diagnostics.rs
git commit -m "feat: stage auditable diagnostics exports"
```

---

### Task 9：严格限制 V1 固定来源、JSON 复杂度与无秘密 preview

**Files:**
- Create: `apps/desktop/src-tauri/src/domain/legacy.rs`
- Create: `apps/desktop/src-tauri/src/application/legacy.rs`
- Modify: `apps/desktop/src-tauri/src/domain/mod.rs`
- Modify: `apps/desktop/src-tauri/src/application/mod.rs`
- Modify: `apps/desktop/src-tauri/src/platform/native_storage.rs`
- Modify: `apps/desktop/src-tauri/src/platform/fake_storage.rs`
- Test: same files

**Interfaces:**
- Consumes: fixed legacy path、Task 2 network rules、SecureStorage read-only API。
- Produces:

```rust
pub const LEGACY_MAX_BYTES: u64 = 256 * 1024;
pub const LEGACY_MAX_DEPTH: usize = 16;
pub const LEGACY_MAX_NODES: usize = 512;
pub const LEGACY_MAX_STRING_BYTES: usize = 8 * 1024;

pub struct LegacyService {
    platform: PlatformId,
    storage: Arc<dyn SecureStorage>,
    logger: Arc<dyn AuditLogger>,
}
impl LegacyService {
    pub fn discover(&self) -> LegacyConfigDiscovery;
    pub fn preview(&self) -> Result<LegacyConfigPreview, LegacyError>;
}
pub fn parse_legacy_config(bytes: &[u8]) -> Result<LegacyConfigPreview, LegacyParseError>;
```

- [ ] **Step 1 (2–5 min, RED): 写平台来源测试**

Windows fake 只返回 `%APPDATA%\ai-coding-installer\config.json` 的 fixed role；macOS 返回 `noSourceForPlatform` 且 storage read count=0。

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml legacy_source_is_fixed_on_windows_and_absent_on_macos
```

Expected: compile FAIL，LegacyService 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现只读 discovery**

Discovery 只查 nofollow metadata：

- missing → notFound；
- regular <= limit → available；
- symlink/reparse/hardlink → unsafePath；
- permissions/IO → unreadable；
- display path 固定 `%APPDATA%\ai-coding-installer\config.json`，不返回 expanded path。

不得读取 content、创建目录、备份或修改 V1。

- [ ] **Step 4 (2–5 min, RED): 写编码/大小/JSON 限制测试**

覆盖：

```text
valid UTF-8
optional UTF-8 BOM
invalid UTF-8
UTF-16LE/BE BOM
256KiB exact
256KiB + 1
depth 16 / 17
nodes 512 / 513
string 8192 / 8193 bytes
duplicate keys at every nesting level
trailing JSON
top-level array/null
```

- [ ] **Step 5 (2–5 min, GREEN): 实现 strict JSON visitor**

使用 `serde_json::Deserializer::from_slice` 的默认 recursion protection，外加自定义 `DeserializeSeed`：

- 每次 map key 放入 `BTreeSet`，重复即拒绝；
- 显式 depth/node/string counters；
- `deserializer.end()` 拒绝 trailing data；
- 不调用 `disable_recursion_limit`；
- parse 后用 iterative drop/visitor，避免恶意深树 drop。

不得把 parse error 原文返回 IPC/log。

- [ ] **Step 6 (2–5 min, RED): 写 V1 已知字段映射**

fixture：

```json
{
  "installNetwork": {
    "mode": "manual_proxy",
    "proxyUrl": "http://127.0.0.1:7890",
    "npmRegistry": "npmmirror",
    "customNpmRegistry": null
  },
  "ccswitchPath": "C:\\secret\\ccSwitch.exe",
  "ccswitchDownloadSources": [{"url":"https://evil.test/a.exe"}],
  "subscriptionPageUrl": "https://secret.test",
  "unknownFuture": {"token":"secret"}
}
```

Expected preview：

```text
NetworkMode eligible “手动代理”
ManualProxy eligible “http://127.0.0.1:7890”, selectedByDefault=false
NpmRegistry eligible “官方源不可用时允许 npmmirror”, selectedByDefault=true
ignored categories: ccSwitch, subscriptionPage, customDownloadSource, unknown
unknownFieldCount=1
secretValueCount>=1
```

不得含 ccSwitch path/source/subscription/unknown value。

- [ ] **Step 7 (2–5 min, GREEN): 实现字段级 preview**

规则：

- `none` → Direct，`system_proxy` → SystemProxy，`manual_proxy` → ManualProxy；
- manual proxy 经 Task 2 validator；credentials → RejectedSecret/display “含凭据，未显示”；
- V1 `default` → OfficialOnly；
- V1 `npmmirror` → OfficialThenNpmmirror；
- V1 `custom` 只有 custom URL 规范化后精确等于两个内置 URL 才映射；其他 IgnoredUnsupported，不回显 URL；
- proxy 永远 `selectedByDefault=false`；
- 忽略 category 去重并按 enum 固定顺序；
- unknown 只计数，不返回 field name/value。

- [ ] **Step 8 (2–5 min, RED→GREEN): 全树 secret 计数**

对所有 string 值运行只检测不回显的 `SecretDetector`；API key、token、Authorization、cookie、proxy credentials 只增加 `secret_value_count`。计数 checked add，overflow/limit 返回 invalid。

- [ ] **Step 9 (2–5 min, RED→GREEN): 不修改 V1**

记录 preview 前后 V1 bytes、mtime、file identity 完全相同；FakeStorage write/remove/replace counters 均 0。源码无 `apply/import/migrate_legacy` production function。

- [ ] **Step 10 (2–5 min, REFACTOR): 恶意 preview corpus**

覆盖 custom URL confusion、unicode hostname、percent userinfo、unknown field 10k、array bomb、secret in key/value、Windows path traversal；输出 JSON 不含输入 secret/path。Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain::legacy
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml application::legacy
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS。

- [ ] **Step 11 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/domain/legacy.rs apps/desktop/src-tauri/src/domain/mod.rs apps/desktop/src-tauri/src/application/legacy.rs apps/desktop/src-tauri/src/application/mod.rs apps/desktop/src-tauri/src/platform
git commit -m "feat: preview fixed-location V1 settings"
```

---

### Task 10：接通窄 IPC、service graph 和零扩权静态门禁

**Files:**
- Modify: `apps/desktop/src-tauri/src/api/commands.rs`
- Modify: `apps/desktop/src-tauri/src/api/events.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/tests/security_configuration.rs`
- Test: same files

**Interfaces:**
- Consumes: Settings/Diagnostics/Legacy services。
- Produces six个 command：

```text
bootstrap
detect_tools
update_settings
prepare_diagnostics
export_diagnostics
preview_legacy_config
```

- [ ] **Step 1 (2–5 min, RED): 写 command core 测试**

普通函数：

```rust
update_settings_inner(&SettingsService, UpdateSettingsRequest)
prepare_diagnostics_inner(&DiagnosticsService)
export_diagnostics_inner(&DiagnosticsService, ExportDiagnosticsRequest)
preview_legacy_config_inner(&LegacyService)
```

断言只返回 DTO/稳定 `CommandError`。

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml slice_two_command_cores_use_typed_requests
```

Expected: compile FAIL，新 command 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现薄 Tauri commands**

attribute wrapper 不做业务、不接受 path/URL/JSON String；`preview_legacy_config` 无参数。Error mapping 必须使用 Task 1 codes/field，不 `format!("{error:?}")`。

- [ ] **Step 4 (2–5 min, RED→GREEN): 实现 settings event sink**

`TauriSettingsEventSink` 只：

```rust
app_handle.emit("settings.changed", event)
```

payload metadata-only。现有 `detection.changed` 不改名。

- [ ] **Step 5 (2–5 min, RED): 写 setup graph 测试/静态断言**

要求唯一实例：

```text
NativeStoragePlatform
Redactor
LocalLogStore (或安全 degraded logger)
AtomicConfigStore
SnapshotStore
BootstrapService
DetectionService
SettingsService
DiagnosticsService
LegacyService
```

logger 安全打开失败时不得写不安全 fallback；snapshot privacy status=readOnlyUnsafePath/unavailable，设置/诊断相应禁用，但 app 可启动展示恢复文案。

- [ ] **Step 6 (2–5 min, GREEN): 组装 setup**

Tauri `app.path().app_data_dir()`/`download_dir()` 只在 setup 解析，传给 native storage。启动顺序：

1. storage paths；
2. logger；
3. config load/recovery；
4. legacy discovery（metadata only）；
5. SnapshotStore schema 3；
6. services/sinks；
7. manage state。

启动不自动 preview V1、不 prepare diagnostics、不发网络。

- [ ] **Step 7 (2–5 min, RED): 扩展安全配置精确测试**

测试必须断言：

```rust
assert_eq!(commands, [
  "bootstrap", "detect_tools", "update_settings",
  "prepare_diagnostics", "export_diagnostics", "preview_legacy_config"
]);
assert_eq!(capability_permissions, SLICE_ONE_EXACT_PERMISSIONS);
assert_eq!(csp, SLICE_ONE_EXACT_CSP);
```

并拒绝 command 名含 `install|upgrade|repair|uninstall|apply|import|execute|download|upload|open_path`。

- [ ] **Step 8 (2–5 min, GREEN): 保持 capability/CSP**

`capabilities/main.json` 和 `tauri.conf.json` 不修改。若编译提示需要 plugin，改 Rust 设计而不是扩权。

- [ ] **Step 9 (2–5 min, RED→GREEN): dependency/endpoint guard**

扫描 Cargo/package/source：

- 无 network/database/async/plugin 依赖；
- production 仅 `domain/network.rs` 两个 allowlist URL；
- 无 telemetry/analytics/crash upload endpoint；
- diagnostics command request 没有 `String destination/path`；
- legacy 没有 apply command；
- config/log/diagnostics modules 无 `std::process::Command`。

- [ ] **Step 10 (2–5 min, REFACTOR): API 回归**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml api
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test security_configuration
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
git diff -- apps/desktop/src-tauri/capabilities/main.json apps/desktop/src-tauri/tauri.conf.json
```

Expected: tests PASS，最后一条无输出。

- [ ] **Step 11 (2–5 min): 提交**

```bash
git add apps/desktop/src-tauri/src/api apps/desktop/src-tauri/src/lib.rs apps/desktop/src-tauri/tests/security_configuration.rs
git commit -m "feat: expose safe settings and diagnostics IPC"
```

---

### Task 11：扩展前端 API/snapshot 同步，不复制协议或持久状态

**Files:**
- Modify: `apps/desktop/src/shared/api/client.ts`
- Modify: `apps/desktop/src/shared/api/client.test.ts`
- Modify: `apps/desktop/src/features/detection/useDetectionSnapshot.ts`
- Modify: `apps/desktop/src/features/detection/useDetectionSnapshot.test.tsx`
- Test: same files

**Interfaces:**
- Consumes: generated Slice 2 DTO。
- Produces:

```ts
updateSettings(request: UpdateSettingsRequest): Promise<SettingsSnapshot>
prepareDiagnostics(): Promise<DiagnosticsPlan>
exportDiagnostics(request: ExportDiagnosticsRequest): Promise<DiagnosticsExportResult>
previewLegacyConfig(): Promise<LegacyConfigPreview>
subscribeSnapshotChanged(listener: (signal: SnapshotChangeSignal) => void): Promise<UnlistenFn>
```

Hook 增加：

```ts
updateSettings(request: UpdateSettingsRequest): Promise<SettingsSnapshot | null>
settingsError: CommandErrorCode | null
prepareDiagnostics(): Promise<DiagnosticsPlan | null>
exportDiagnostics(request: ExportDiagnosticsRequest): Promise<DiagnosticsExportResult | null>
previewLegacyConfig(): Promise<LegacyConfigPreview | null>
```

- [ ] **Step 1 (2–5 min, RED): 写 client 精确 invoke 测试**

```ts
expect(invoke).toHaveBeenCalledWith("update_settings", { request });
expect(invoke).toHaveBeenCalledWith("prepare_diagnostics");
expect(invoke).toHaveBeenCalledWith("export_diagnostics", { request });
expect(invoke).toHaveBeenCalledWith("preview_legacy_config");
```

`exportDiagnostics` 只能接收 generated `ExportDiagnosticsRequest`；不存在 `(path: string)` overload。

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
npm run test --workspace @code-ready/desktop -- src/shared/api/client.test.ts
```

Expected: FAIL，新 API 不存在。

- [ ] **Step 3 (2–5 min, GREEN): 实现 client**

只从 `generated/index.ts` 导入 DTO；command/event 名均为常量 literal。扩展 `COMMAND_ERROR_CODES` 与 generated enum 一致，不把 unknown object message 显示。

- [ ] **Step 4 (2–5 min, RED): 写双 event subscribe 测试**

`subscribeSnapshotChanged` 先后 listen：

```text
detection.changed
settings.changed
```

任一 listener 建立失败时调用已建立 unlisten 并 reject；组合 unlisten 恰调用两个一次。回调只转交 sequence/snapshotVersion，不传 proxy/设置事实。

- [ ] **Step 5 (2–5 min, GREEN): 实现组合 subscription**

`SnapshotChangeSignal` 只能写成 generated 类型的 `Pick<...>` union，不重写 wire fields：

```ts
type SnapshotChangeSignal =
  | Pick<DetectionEventEnvelope, "sequence" | "snapshotVersion">
  | Pick<SettingsEventEnvelope, "sequence" | "snapshotVersion">;
```

- [ ] **Step 6 (2–5 min, RED): 扩展 hook 恢复测试**

覆盖：

1. settings event 连续/重复/gap 都走 bootstrap；
2. update command response 不直接拼 snapshot，等待 refresh/bootstrap；
3. update success 但 event 丢失，显式 refresh 得到新 revision；
4. revision conflict 保留旧 draft/snapshot；
5. diagnostics/legacy command error 不清空 snapshot；
6. stale bootstrap 不能覆盖更新后的 revision。

- [ ] **Step 7 (2–5 min, GREEN): 实现统一 snapshot truth**

保留现有 refresh coalescing/highest sequence 逻辑；把 subscription 替换为双 event。Mutation helper 调用 command 后 `await refresh()`，但只有 bootstrap response 更新 snapshot。

- [ ] **Step 8 (2–5 min, RED→GREEN): persisted onboarding hook**

提供：

```ts
completeOnboarding(): Promise<void>
```

它从当前 generated settings 构造完整 `UpdateSettingsRequest`，只把 `onboardingCompleted=true`，携带 expectedRevision。不得 localStorage/sessionStorage。

- [ ] **Step 9 (2–5 min, REFACTOR): 无手写 DTO 守卫**

Run:

```bash
git grep -n "interface SettingsSnapshot\\|type NetworkMode\\|interface DiagnosticsPlan\\|interface LegacyConfigPreview" -- apps/desktop/src ':!apps/desktop/src/shared/api/generated'
npm run test --workspace @code-ready/desktop -- src/shared/api/client.test.ts src/features/detection/useDetectionSnapshot.test.tsx
npm run typecheck
npm run lint
```

Expected: grep 无输出，其余 PASS。

- [ ] **Step 10 (2–5 min): 提交**

```bash
git add apps/desktop/src/shared/api/client.ts apps/desktop/src/shared/api/client.test.ts apps/desktop/src/features/detection
git commit -m "feat: synchronize configuration snapshots"
```

---

### Task 12：实现非技术化设置 UX 和中文文案

**Files:**
- Create: `apps/desktop/src/features/settings/SettingsPanel.tsx`
- Create: `apps/desktop/src/features/settings/SettingsPanel.test.tsx`
- Modify: `apps/desktop/src/features/onboarding/Onboarding.tsx`
- Modify: `apps/desktop/src/features/onboarding/Onboarding.test.tsx`
- Modify: `apps/desktop/src/features/dashboard/StatusCenter.tsx`
- Modify: `apps/desktop/src/features/dashboard/StatusCenter.test.tsx`
- Modify: `apps/desktop/src/shared/i18n/zh-CN.ts`
- Modify: `apps/desktop/src/shared/i18n/zh-CN.test.ts`
- Test: same files

**Interfaces:**
- Consumes: snapshot settings/health、hook update/complete actions。
- Produces: editable draft → Rust validation/revision update；持久化 onboarding。

- [ ] **Step 1 (2–5 min, RED): 写中文 key 完整性测试**

至少锁定：

```text
settings.title = 设置
settings.network.title = 网络设置
settings.network.direct = 直接连接（不使用代理）
settings.network.system = 使用系统代理
settings.network.systemNote = Code-Ready 不会修改系统代理。
settings.network.manual = 使用手动 HTTP(S) 代理
settings.network.manualLabel = 代理地址
settings.network.manualHelp = 只支持不含用户名和密码的 http:// 或 https:// 地址。
settings.registry.officialOnly = 仅使用 npm 官方源（推荐）
settings.registry.fallback = 官方源不可用时，允许使用 npmmirror
settings.scopeNotice = 本切片只保存设置，不会测试连接，也不会安装、升级或修复工具。
settings.save = 保存设置
settings.saved = 设置已保存
settings.revisionConflict = 设置已在别处更新，请刷新后确认再保存。
settings.invalidProxy = 代理地址无效。请使用不含凭据的 HTTP(S) 地址。
settings.readOnly = 设置文件需要人工处理，当前不能保存。
navigation.settingsPrivacy = 设置与隐私
navigation.status = 环境状态
```

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
npm run test --workspace @code-ready/desktop -- src/shared/i18n/zh-CN.test.ts
```

Expected: FAIL，Slice 2 keys 缺失。

- [ ] **Step 3 (2–5 min, GREEN): 添加 typed 文案**

所有用户可见文本进入 resource；不得出现 “WAL/JSON/schema/fsync/IPC/registry URL”。平台/path 数字继续由 formatter 输出。

- [ ] **Step 4 (2–5 min, RED): 写设置初始/草稿测试**

render generated snapshot：

- 三个 network radio；
- manual 未选时 input 不显示；
- officialOnly 默认；
- 切 manual 显示 input；
- 切回 direct 清空 draft manual URL；
- 未改时 save disabled；
- scope notice 可见。

- [ ] **Step 5 (2–5 min, GREEN): 实现 controlled draft**

`SettingsPanel` local state 只作未保存 draft，不是事实；snapshot revision 变化且用户无 dirty draft 时重置，有 dirty draft 时显示“设置已更新，请确认”而不静默覆盖。

- [ ] **Step 6 (2–5 min, RED): 写提交 payload 测试**

manual 输入 `" HTTP://Proxy.Example:8080/ "` 时前端原样提交完整 request（不自行规范化）；后端 mock 返回 normalized snapshot，刷新后 UI 显示 normalized value。这样 Rust 保持 validator 权威。

- [ ] **Step 7 (2–5 min, GREEN): 实现保存/错误**

保存 button pending disabled；错误只按 `CommandErrorCode/field` 映射中文：

- invalidSettings/manualProxyUrl；
- revisionConflict；
- configReadOnly/unsafePath；
- internal。

不显示 rejected input/debug error。

- [ ] **Step 8 (2–5 min, RED→GREEN): 系统代理与 registry 解释**

测试选择 system 显示“不修改系统代理”；fallback 显示“仅白名单”，页面没有 custom URL input、“测试连接”button 或真实 registry URL。

- [ ] **Step 9 (2–5 min, RED): 持久化 onboarding 流程**

Onboarding 完成 button 调用 `completeOnboarding()`，pending 时 disabled；success 后进入 dashboard。App reload snapshot `settings.onboardingCompleted=true` 时直接 dashboard，即使 detectionRun 为 null/failed；false 时仍 onboarding。

- [ ] **Step 10 (2–5 min, GREEN): 更新 Onboarding/App 入口语义**

不再用 detectionRun completed 决定首次模式；它只决定检测步骤。completion 失败显示中文 alert 并保留解释页，不进入 dashboard。

- [ ] **Step 11 (2–5 min, RED→GREEN): 导航与可访问性**

StatusCenter 增加“设置与隐私”button，由 App mode 切换；Settings 返回“环境状态”。测试：

- 单一 h1；
- fieldset/legend/radio label；
- error alert、saved status；
- visible focus；
- 320px/200% 单列；
- 无安装/测试网络/apply V1 操作。

- [ ] **Step 12 (2–5 min, REFACTOR): 前端验证**

```bash
npm run test --workspace @code-ready/desktop -- src/features/settings/SettingsPanel.test.tsx
npm run test --workspace @code-ready/desktop -- src/features/onboarding/Onboarding.test.tsx src/features/dashboard/StatusCenter.test.tsx
npm run test --workspace @code-ready/desktop -- src/shared/i18n/zh-CN.test.ts
npm run typecheck
npm run lint
```

Expected: PASS。

- [ ] **Step 13 (2–5 min): 提交**

```bash
git add apps/desktop/src/features/settings apps/desktop/src/features/onboarding apps/desktop/src/features/dashboard apps/desktop/src/shared/i18n
git commit -m "feat: add safe network settings"
```

---

### Task 13：实现诊断清单/明确导出与 V1 只读预览 UX

**Files:**
- Create: `apps/desktop/src/features/diagnostics/DiagnosticsPanel.tsx`
- Create: `apps/desktop/src/features/diagnostics/DiagnosticsPanel.test.tsx`
- Create: `apps/desktop/src/features/legacy/LegacyConfigPreview.tsx`
- Create: `apps/desktop/src/features/legacy/LegacyConfigPreview.test.tsx`
- Modify: `apps/desktop/src/app/App.tsx`
- Modify: `apps/desktop/src/app/App.test.tsx`
- Modify: `apps/desktop/src/app/styles.css`
- Modify: `apps/desktop/src/shared/i18n/zh-CN.ts`
- Modify: `apps/desktop/src/shared/i18n/zh-CN.test.ts`
- Test: same files

**Interfaces:**
- Consumes: generated manifest/discovery/preview/result。
- Produces: prepare → audit → explicit export；discover → click preview，无 apply。

- [ ] **Step 1 (2–5 min, RED): 添加诊断/V1 中文 keys**

精确中文：

```text
diagnostics.title = 隐私与诊断
diagnostics.zeroTelemetry = 日志只保存在本机，不会自动上传。
diagnostics.retention = 日志最多保留 30 天或 100 MiB，先达到任一上限就清理最旧内容。
diagnostics.prepare = 准备诊断包
diagnostics.preparing = 正在本机准备并再次脱敏…
diagnostics.manifest = 导出清单
diagnostics.reviewConfirm = 我已查看清单
diagnostics.export = 导出到下载文件夹
diagnostics.exported = 诊断包已导出到下载文件夹。
diagnostics.expired = 清单已过期，请重新准备。
diagnostics.diskFull = 下载文件夹空间不足，未导出文件。
diagnostics.nameConflict = 同名文件已存在，未覆盖。请稍后重新准备。
legacy.title = 旧版配置
legacy.available = 找到 Windows 旧版配置，可安全预览。
legacy.notFound = 未找到旧版配置。
legacy.noSourceMac = macOS 没有批准的旧版配置来源。
legacy.preview = 预览可识别设置
legacy.previewOnly = 本版本只显示预览，不会应用或修改旧版配置。
legacy.proxyDefaultOff = 手动代理若未来支持导入，也会默认不选择。
legacy.secretRejected = 检测到可能包含凭据的值，已隐藏且不会导入。
legacy.ignored = 其他旧版字段已忽略。
```

- [ ] **Step 2 (2–5 min, RED): 写 prepare 前状态测试**

页面显示 zero telemetry/retention、“准备诊断包”；没有 export button、上传、打开日志目录、任意路径 input。

- [ ] **Step 3 (2–5 min, RED): 运行**

```bash
npm run test --workspace @code-ready/desktop -- src/features/diagnostics/DiagnosticsPanel.test.tsx
```

Expected: FAIL，component 不存在。

- [ ] **Step 4 (2–5 min, GREEN): 实现 prepare state**

点击后调用 `prepareDiagnostics()`；pending status；成功显示：

- created local time；
- entries table：archive path、media type、human byte size、SHA-256 前 12 位（完整 hash accessible detail）；
- total；
- checkbox。

不显示 staging absolute path。

- [ ] **Step 5 (2–5 min, RED): 写明确导出测试**

导出 button 仅在 plan 存在且 checkbox checked 时 enabled；click 精确发送：

```ts
{
  planId: plan.id,
  expectedManifestSha256: plan.manifestSha256,
  target: "downloads",
}
```

成功只显示 `result.displayLocation`/safe filename。

- [ ] **Step 6 (2–5 min, GREEN): 实现 export/error**

错误按 stable code 映射；失败保留 manifest（expired 除外），不假装成功、不自动选择其他目录、不调用 opener。

- [ ] **Step 7 (2–5 min, RED): 写 V1 三平台/状态测试**

实际两个平台语义：

- Windows available → preview button；
- Windows notFound/unsafe/unreadable → 无 preview 或稳定说明；
- mac noSourceForPlatform → 明确中文、无 button；
- 页面始终显示 previewOnly；
- DOM 中不存在“应用/导入/迁移/删除旧配置”button。

- [ ] **Step 8 (2–5 min, GREEN): 实现 LegacyConfigPreview**

用户点击才 `previewLegacyConfig()`；field rows 只渲染 generated displayValue；RejectedSecret 显示通用文案；ignored categories 用本地 enum mapping，unknown 只显示数量，不显示 field names。

- [ ] **Step 9 (2–5 min, RED→GREEN): secret DOM scan**

mock preview 不应包含 raw secret；另外把 client reject object 填入 secret debug string，UI 仍只显示稳定中文。断言 DOM/console spy 均无 secret。

- [ ] **Step 10 (2–5 min, RED→GREEN): App 组合**

Settings/Privacy screen 同时包含 `SettingsPanel`、`DiagnosticsPanel`、`LegacyConfigPreview`；从 dashboard 可进入/返回。bootstrap config health/privacy status 为 degraded 时显示只读解释并禁用对应 action。

- [ ] **Step 11 (2–5 min, RED→GREEN): 可访问性/布局**

manifest 用 caption/table headers；checkbox 有完整 label；pending `role=status`；error `role=alert`；hash/code `overflow-wrap:anywhere`；320px table 改 stacked list；200% zoom 无横向页面溢出。

- [ ] **Step 12 (2–5 min, REFACTOR): 前端完整回归**

```bash
npm run test --workspace @code-ready/desktop -- src/features/diagnostics/DiagnosticsPanel.test.tsx
npm run test --workspace @code-ready/desktop -- src/features/legacy/LegacyConfigPreview.test.tsx
npm run test --workspace @code-ready/desktop -- src/app/App.test.tsx
npm run typecheck
npm run lint
npm run build
```

Expected: PASS，生产 bundle 无远程资源。

- [ ] **Step 13 (2–5 min): 提交**

```bash
git add apps/desktop/src/features/diagnostics apps/desktop/src/features/legacy apps/desktop/src/app apps/desktop/src/shared/i18n
git commit -m "feat: add diagnostics and V1 preview UX"
```

---

### Task 14：Fake 双平台纵向、恶意输入、契约/静态安全和文档收口

**Files:**
- Create: `apps/desktop/src-tauri/tests/slice_two_flow.rs`
- Modify: `apps/desktop/src-tauri/tests/security_configuration.rs`
- Modify: `apps/desktop/src-tauri/tests/detection_flow.rs`
- Modify: `docs/testing/v2-platform-matrix.md`
- Modify: `README.md`
- Test: full Rust/React/root suites

**Interfaces:**
- Consumes: 完整 service graph + React fixtures。
- Produces: hosted 双平台门禁、人工干净机矩阵、明确延期边界。

- [ ] **Step 1 (2–5 min, RED): 写 Windows/macOS fake 纵向测试**

```rust
#[test]
fn slice_two_fake_flow_is_equivalent_and_side_effect_bounded() {
    for platform in [PlatformId::WindowsX64, PlatformId::MacosArm64] {
        let h = SliceTwoHarness::new(platform);
        let initial = h.bootstrap();
        assert_eq!(initial.schema_version, 3);
        assert!(!initial.privacy.telemetry_enabled);

        let updated = h.update(manual_settings(initial.settings.revision)).unwrap();
        assert_eq!(updated.network.manual_proxy_url.as_deref(), Some("http://127.0.0.1:7890"));

        let plan = h.prepare_diagnostics().unwrap();
        assert!(!plan.manifest.entries.is_empty());
        let result = h.export(plan).unwrap();
        assert!(result.display_location.starts_with("下载/"));

        assert_eq!(h.network_requests(), 0);
        assert_eq!(h.process_requests(), 0);
        assert_eq!(h.privilege_requests(), 0);
        assert_eq!(h.install_requests(), 0);
    }
}
```

Windows 另断言 V1 available/preview；mac 断言 noSource/read count 0。

- [ ] **Step 2 (2–5 min, RED): 运行**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test slice_two_flow
```

Expected: compile FAIL，harness/constructors 未完整公开。

- [ ] **Step 3 (2–5 min, GREEN): 暴露最小构造器并跑通**

只公开 trait 注入所需 constructors；不公开内部 mutex、任意 path setter、raw log writer 或 config bypass。

- [ ] **Step 4 (2–5 min, RED→GREEN): 恶意输入总矩阵**

集成测试串联：

```text
proxy credentials/SOCKS/path/query/fragment
registry custom URL
config duplicate/unknown/checksum mismatch/future schema
config/log/download symlink/reparse/hardlink
atomic failpoints
log secret/control/newline/oversize
ZIP traversal/duplicate/oversize
Downloads collision/TOCTOU/permission/ENOSPC
V1 invalid UTF/BOM/oversize/depth/nodes/duplicates/secrets
stale settings revision
stale/expired diagnostics plan
```

每项断言稳定 code、无 secret/path error、无 fallback mutation。

- [ ] **Step 5 (2–5 min, RED): 写静态 capability/command/依赖精确门禁**

`security_configuration.rs` 断言：

1. handler 恰六个批准 command；
2. event 恰 `detection.changed/settings.changed`；
3. CSP/capability 与 `8ada8ca` 精确值相同；
4. Cargo 新依赖集合恰是最小审计表，目标依赖 feature 恰匹配；
5. 无 SQLite/network/async/Tauri plugin；
6. package.json runtime dependency 不变；
7. production 外部 URL 恰两个 allowlist literal；
8. no import/apply/install/update/repair/uninstall command；
9. diagnostics target enum 只有 downloads；
10. generated DTO 无 drift。

- [ ] **Step 6 (2–5 min, GREEN): 修正实现而非放宽门禁**

任何门禁 RED 优先删除多余 API/依赖/endpoint。不得用测试 allowlist 加入实现中不必要的字符串。

- [ ] **Step 7 (2–5 min, RED→GREEN): binding drift 红灯演练**

临时破坏一个新 generated file：

```bash
printf '\n' >> apps/desktop/src/shared/api/generated/SettingsSnapshot.ts
npm run check:bindings
```

Expected: 非零并显示 diff。立即恢复：

```bash
npm run generate:bindings
npm run check:bindings
```

Expected: 0；临时破坏不得提交。执行记录注明实际观察到 RED。

- [ ] **Step 8 (2–5 min, REFACTOR): 更新平台矩阵文档**

新增 Slice 2 段落，明确 hosted CI 只证明：

- Windows Server x64/macOS arm64 compile/test；
- fake storage/config/log/diagnostics/V1 contracts；
- macOS target 13.0 compile floor。

真实机器仍需：

- Windows 11 x64：APPDATA/Downloads ACL、reparse、ReplaceFileW、磁盘满、文件冲突、V1 各状态；
- macOS 13+ Apple Silicon：0700/0600、O_NOFOLLOW、rename/dir fsync、Downloads symlink/磁盘满；
- 两端 crash failpoint/强制退出后 primary/backup 恢复；
- 日志 30 天/100 MiB、并发/截断尾行；
- 导出 ZIP 人工清单/脱敏抽查；
- 网络观察确认零 DNS/HTTP/telemetry/upload；
- UI 200%/键盘/中文文案。

未完成前只能写 “hosted compile/test coverage”。

- [ ] **Step 9 (2–5 min): 更新 README**

当前状态改为 Slice 2：已支持安全设置、日志、诊断和 V1 preview；明确：

- 不发网络；
- 不测试代理；
- 不 apply V1；
- 不安装/升级/修复/卸载；
- SQLite 未引入，原子文件选择理由；
- Downloads 固定导出与不自动上传。

- [ ] **Step 10 (2–5 min): 运行针对性 Rust tests**

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain::network
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test config_recovery
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml infrastructure::redaction
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml infrastructure::log_store
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test privacy_diagnostics
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml domain::legacy
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test slice_two_flow
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test security_configuration
```

Expected: 全部 PASS。

- [ ] **Step 11 (2–5 min): 运行前端针对性 tests**

```bash
npm run test --workspace @code-ready/desktop -- src/features/settings/SettingsPanel.test.tsx
npm run test --workspace @code-ready/desktop -- src/features/diagnostics/DiagnosticsPanel.test.tsx
npm run test --workspace @code-ready/desktop -- src/features/legacy/LegacyConfigPreview.test.tsx
npm run test --workspace @code-ready/desktop -- src/features/detection/useDetectionSnapshot.test.tsx
npm run test --workspace @code-ready/desktop -- src/app/App.test.tsx
```

Expected: 全部 PASS。

- [ ] **Step 12 (2–5 min): 运行完整本地验证**

```bash
git status --short
npm ci
npm run check
npm run build
npm run tauri:build
git diff --check
```

Expected:

- TS、ESLint、Vitest、Rust fmt/clippy/test、binding drift PASS；
- Vite/current host unsigned Tauri build PASS；
- diff check 无输出；
- status 只含本任务 docs/tests 预期变更，无 dist/target/log/config/zip/temp。

- [ ] **Step 13 (2–5 min): 运行人工范围扫描**

```bash
rg -n -i "rusqlite|sqlite|reqwest|ureq|hyper|tokio|async-std|tauri-plugin-(dialog|opener|fs|http|shell|process|updater)" apps package.json
rg -n -i "install|upgrade|repair|uninstall|apply_legacy|import_legacy|download|upload" apps/desktop/src-tauri/src/api apps/desktop/src/shared/api/client.ts
rg -n "https?://" apps/desktop/src-tauri/src apps/desktop/src ':!apps/desktop/src-tauri/src/domain/network.rs'
git diff -- apps/desktop/src-tauri/capabilities/main.json apps/desktop/src-tauri/tauri.conf.json
git grep -n "interface SettingsSnapshot\\|interface DiagnosticsPlan\\|interface LegacyConfigPreview" -- apps/desktop/src ':!apps/desktop/src/shared/api/generated'
```

Expected:

- 第一条仅安全测试禁止词 fixture；
- 第二条无 production action；
- 第三条无 production endpoint（测试 fixtures 需人工确认）；
- CSP/capability 无 diff；
- 无手写 wire DTO。

- [ ] **Step 14 (2–5 min): 请求独立代码审查**

使用 `requesting-code-review`，重点：

- 原子替换各 crash point 是否至少一份有效；
- nofollow/reparse/ACL 是否存在 check-then-open TOCTOU；
- checksum canonicalization/duplicate key/future schema；
- proxy userinfo/IPv6/IDNA/percent encoding；
- log first-line minimization 与 double redaction；
- rotation exact boundary/concurrency/partial tail；
- ZIP name/size/hash/collision/ENOSPC；
- V1 raw/secret 是否可能进入 DTO/log；
- snapshot durable commit/event ordering；
- command/capability/network/install surface。

P0/P1 修复后重跑相关 RED→GREEN；低优先级问题记录处置。

- [ ] **Step 15 (2–5 min): 提交 Slice 2 收口**

```bash
git add apps/desktop/src-tauri/tests docs/testing/v2-platform-matrix.md README.md
git commit -m "test: verify Slice 2 privacy boundaries"
git status --short --branch
```

Expected: 工作树干净；不 push、不 PR、不修改 main。

---

## 设置/恢复测试矩阵

| Case | Expected durable state | Expected health/error |
| --- | --- | --- |
| first start | revision 1 defaults + valid checksum | healthy |
| current V1 doc | unchanged bytes on read | healthy |
| internal V0 doc | migrated V1, V0 backup retained | migrated |
| interrupted temp write | old primary | healthy |
| backup replaced, primary not yet replaced | old primary/backup | healthy |
| primary replaced, dir sync interrupted | old or new complete doc | healthy/recoveredBackup |
| corrupt primary + valid backup | backup restored | recoveredBackup |
| corrupt primary + no valid backup | defaults, evidence quarantined | recoveredDefaults |
| future schema | no overwrite/downgrade | readOnlyFutureVersion |
| symlink/reparse/hardlink | no follow/write | readOnlyUnsafePath |
| two writers same revision | one revision+1 | second revisionConflict |
| disk full | previous valid doc | internal retryable |

## 日志/诊断测试矩阵

| Case | Required behavior |
| --- | --- |
| record exactly fits 5 MiB | same file |
| record exceeds by 1 byte | next sequence |
| UTC day changes | new day file |
| age exactly 30 days | retained |
| age 31 days | deleted before write |
| total exactly 100 MiB | retained |
| total exceeds by 1 byte | oldest closed logs deleted |
| concurrent writes | valid non-interleaved JSONL |
| crash truncates tail | tail replaced by safe discard record on export |
| secret/path/query | absent after write and second export pass |
| prepare only | private staging + manifest, no Downloads file |
| export unchecked | disabled in UI |
| target exists | no overwrite, stable conflict |
| target becomes symlink | unsafePath, no fallback |
| ENOSPC mid-copy | partial removed, staging retained |
| plan expired/hash differs | no export |
| successful export | exact hash/size, safe display location |

## V1 preview 测试矩阵

| Input | Preview |
| --- | --- |
| Windows no file | notFound |
| macOS | noSourceForPlatform, zero read |
| valid direct/default | eligible direct + official |
| valid system/npmmirror | eligible system + fallback |
| valid manual HTTP(S) | eligible proxy, selectedByDefault=false |
| SOCKS/manual credentials | rejectedInvalid/rejectedSecret, no raw |
| custom exact official/npmmirror URL | maps to corresponding allowlist |
| custom other URL | ignoredUnsupported |
| ccSwitch/subscription/custom sources | category only |
| unknown fields | count only |
| secret anywhere | count only, never value |
| oversize/invalid encoding/depth/duplicate | stable invalid/tooLarge |
| preview completes | V1 bytes/mtime/identity unchanged |

## Slice 2 完成定义

只有同时满足以下条件，执行者才能报告 Slice 2 本地实现完成：

1. 设置存储没有 SQLite/database dependency，版本化原子 JSON、checksum、backup、migration、quarantine/recovery 测试通过。
2. Windows/macOS secure storage fake contract 覆盖 fixed path、nofollow/reparse、权限、TOCTOU、atomic replace、disk full；native hosted tests 只作平台能力证明。
3. 代理只接受 normalized credential-free HTTP(S) authority；SOCKS/auth/path/query/fragment 全拒绝。
4. registry 只有 officialOnly/officialThenNpmmirror，production 只有两个 allowlist URL，无 live network。
5. durable settings update 使用 expectedRevision，commit→snapshot→event 顺序通过；event 丢失可 bootstrap 恢复。
6. onboarding completion 跨进程持久化，前端无 localStorage。
7. 日志类型化、写前脱敏、导出二次脱敏；secret/path/query corpus 与生成 corpus 无泄漏。
8. 日志 5 MiB/30 天/100 MiB 精确边界、并发和崩溃尾行测试通过。
9. 诊断先 manifest 后明确 export；固定 Downloads，无 overwrite、无 arbitrary path、无自动上传。
10. ZIP entry/path/count/size/hash/权限，Downloads collision/symlink/TOCTOU/permission/ENOSPC/expiry 全部测试通过。
11. Windows V1 只读固定来源；macOS no source；大小/编码/depth/node/string/duplicate/trailing 限制通过。
12. V1 preview 不回显未知值/secret，不修改 V1；无 apply/import command。
13. 设置、诊断、V1 中文 UX、键盘/语义/200%/320px 测试通过。
14. Rust/ts-rs 是所有 IPC 唯一来源，binding drift 红灯实际观察，最终无漂移。
15. CSP/capability 与 Slice 1 字节级不变；六 command 精确 allowlist；无网络、数据库、async、plugin、安装/权限 surface。
16. `npm run check`、`npm run build`、当前宿主 `npm run tauri:build` 有本轮新鲜成功输出。
17. hosted Windows/macOS CI 未运行前只报告待运行；真实 Windows 11/macOS 13 干净机仍是人工 gate。
18. 工作树干净，提交范围只包含 Slice 2 实现、测试和文档。

## 明确延期边界

- V1 设置的选择、确认和实际 apply；本切片没有 import/apply API。
- 任何安装、升级、修复、卸载、计划、下载、完整性验证或执行。
- 代理连接测试、系统代理读取实现、把代理注入下载器/外部工具。
- npm/npmmirror 实际网络回退、官方 hash/signature 联动；延期 Slice 9。
- arbitrary save path/native save dialog；若未来需要，单独 capability 审查。
- 安装会话 journal、任务快照数据库；原子文件不足时才重新评估 SQLite。
- 日志查看器、打开日志目录、上传诊断、远程支持会话。
- 遥测、崩溃自动上传、主机/设备标识。
- Windows/macOS 签名、公证、公开发布和干净机已通过声明。

## 计划自审记录

- **规范覆盖：** 用户列出的 config schema/migration/atomic/recovery、proxy、registry、redaction/rotation、manifest/ZIP/export、V1 parser/preview、UX、fake adapters、契约和静态安全分别映射 Tasks 1–14。
- **依赖审计：** SQLite 被原子文件替代；新增 production crate 均有单一用途和禁止 feature，JavaScript/runtime/network/database 依赖为零。
- **接口一致：** `SettingsSnapshot.revision`、`UpdateSettingsRequest.expectedRevision`、`SettingsEventEnvelope.settingsRevision`、`DiagnosticsPlan.manifestSha256`、`ExportDiagnosticsRequest.expectedManifestSha256` 在 Rust/TS/UI/测试同名。
- **路径审计：** config/log/staging/V1/Downloads 都通过 `SecureStorage`；没有 command 接受路径，archive path 与 filesystem path 使用不同 newtype。
- **秘密审计：** proxy 可保存但不进入 LogEvent/diagnostics payload；V1 raw/unknown values 不进入 DTO；errors 不含 debug string。
- **并发/崩溃审计：** config failpoint、log mutex/partial tail、diagnostic copy/rename failpoint和 event loss 均有明确 expected state。
- **范围审计：** 无 apply V1、network request、dialog capability、安装/权限/更新；延期写明。
- **占位审计：** 文档不含未决实现项；所有限制、常量、命令、文件、error、测试名和提交边界已固定。
