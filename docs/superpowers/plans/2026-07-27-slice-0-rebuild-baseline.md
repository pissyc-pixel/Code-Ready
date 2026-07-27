# Code-Ready V2 Slice 0：重写基线与契约实施计划

> **执行者：Luna Max。** 开始前必须使用 `executing-plans`、`test-driven-development` 和 `verification-before-completion`。严格按任务顺序执行；每个行为任务都先看到预期失败，再写最小实现。遇到设计冲突时停止并回报，不得自行扩大 Slice 0。

**目标：** 删除 V1 的生产代码骨架（Git 历史和 V1 文档保留），建立可在 Windows x64 与 macOS Apple Silicon 编译的 Tauri 2 + React + TypeScript + Rust 单应用；由 Rust 单向生成 TypeScript IPC 类型；用类型化内置工具注册表和平台适配接口锁定 V2 的基础契约。

**验收结果：**

- 仓库根目录成为 npm workspace，桌面应用位于 `apps/desktop`。
- V1 的根级 `src`、`src-tauri`、`public` 和旧构建配置不再存在。
- Rust 是 IPC DTO 的唯一权威来源，生成文件提交到 Git，CI 会发现漂移。
- 注册表固定表达 WinGet、Git、Node.js/npm、Claude Code、Codex CLI 的双平台策略；npm 是 Node.js 的派生能力，不是独立工具。
- Claude Code 与 Codex CLI 的运行时依赖中没有 Node.js/npm。
- `PlatformAdapter` 隔离平台识别，测试使用 `FakePlatformAdapter`，业务核心无散落的 `cfg`/PowerShell/shell 判断。
- 最小 UI 通过 `get_bootstrap_state` 显示当前平台及注册表摘要。
- Tauri CSP 非空、无远程脚本、无 `unsafe-eval`，capability 不包含 shell、process、fs、http、dialog 或 opener 权限。
- GitHub Actions 在 `windows-2025` x64 与 `macos-15` arm64 上执行完整检查并构建无签名应用。

**架构约束：**

```text
React UI
  └─ shared/api（仅使用生成 DTO）
      └─ Tauri command
          └─ application/bootstrap
              ├─ domain/contracts + domain/tool_registry
              └─ platform/PlatformAdapter
```

`domain` 不依赖 Tauri、React、具体系统命令或网络。Slice 0 不检测、不下载、不安装任何工具，不创建提权 helper，不实现配置/日志/迁移/更新。

**固定工具链：**

- Node.js 24，npm 11。
- React 19.2.8，React DOM 19.2.8。
- TypeScript 7.0.2，Vite 8.1.5，Vitest 4.1.10。
- Tauri CLI 2.11.4，Tauri JS API 2.11.1。
- Rust stable，最低必须满足 `ts-rs` 的 Rust 1.88 要求。
- `ts-rs` 12.0.1。
- macOS deployment target 为 13.0。

---

## Task 1：建立 V2 workspace，移除 V1 生产骨架

**删除：**

- `index.html`
- `public/`
- `src/`
- `src-tauri/`
- `tsconfig.json`
- `tsconfig.node.json`
- `vite.config.ts`
- 旧 `package-lock.json`

**保留：**

- `.gitignore`
- `.vscode/`
- `README.md`
- `RELEASE_NOTES.md`
- `docs/`
- `.git/` 历史与 `main`

**创建：**

- `package.json`
- `apps/desktop/package.json`
- `apps/desktop/index.html`
- `apps/desktop/tsconfig.json`
- `apps/desktop/tsconfig.node.json`
- `apps/desktop/vite.config.ts`
- `apps/desktop/eslint.config.js`
- `apps/desktop/src/vite-env.d.ts`
- `apps/desktop/src/test/setup.ts`

### Step 1：删除旧生产文件并移动纯图标资产

先把 `src-tauri/icons/` 移到 `apps/desktop/src-tauri/icons/`，再删除上面的旧生产树。图标是唯一允许复用的 V1 资产；不得复制旧 React 或 Rust 源码。

### Step 2：创建根 workspace manifest

根 `package.json` 必须满足：

```json
{
  "name": "code-ready-v2",
  "version": "0.0.0",
  "private": true,
  "license": "UNLICENSED",
  "workspaces": ["apps/*"],
  "engines": {
    "node": ">=24",
    "npm": ">=11"
  },
  "scripts": {
    "dev": "npm run tauri:dev --workspace @code-ready/desktop",
    "build": "npm run build --workspace @code-ready/desktop",
    "typecheck": "npm run typecheck --workspace @code-ready/desktop",
    "lint": "npm run lint --workspace @code-ready/desktop",
    "test": "npm run test --workspace @code-ready/desktop",
    "test:rust": "cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml",
    "lint:rust": "cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings",
    "format:check": "cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml --all -- --check",
    "generate:bindings": "cargo run --quiet --manifest-path apps/desktop/src-tauri/Cargo.toml --bin export-bindings",
    "check:bindings": "npm run generate:bindings && git diff --exit-code -- apps/desktop/src/shared/api/generated",
    "tauri:build": "npm run tauri:build --workspace @code-ready/desktop",
    "check": "npm run typecheck && npm run lint && npm run test && npm run format:check && npm run lint:rust && npm run test:rust && npm run check:bindings"
  }
}
```

桌面 workspace 必须保持 `private: true` 和 `license: "UNLICENSED"`，不得添加 LICENSE：

```json
{
  "name": "@code-ready/desktop",
  "version": "0.0.0",
  "private": true,
  "license": "UNLICENSED",
  "type": "module",
  "scripts": {
    "dev:web": "vite --port 1420",
    "build:web": "tsc -b && vite build",
    "build": "npm run build:web",
    "typecheck": "tsc -b --pretty false",
    "lint": "eslint . --max-warnings 0",
    "test": "vitest run",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build --no-bundle"
  }
}
```

依赖必须精确锁定，不使用 `^` 或 `~`：

```text
dependencies:
  @tauri-apps/api 2.11.1
  react 19.2.8
  react-dom 19.2.8

devDependencies:
  @tauri-apps/cli 2.11.4
  @testing-library/jest-dom 7.0.0
  @testing-library/react 16.3.2
  @types/react 19.2.17
  @types/react-dom 19.2.3
  @vitejs/plugin-react 6.0.4
  eslint 10.8.0
  eslint-plugin-react-hooks 7.1.1
  eslint-plugin-react-refresh 0.5.3
  jsdom 29.1.1
  typescript 7.0.2
  typescript-eslint 8.65.0
  vite 8.1.5
  vitest 4.1.10
```

### Step 3：安装依赖并验证空 workspace

Run:

```bash
npm install
npm run typecheck
```

Expected:

- 生成新的根 `package-lock.json`，lockfile 由 npm 11 生成。
- `npm run typecheck` 成功；此时允许应用还没有业务 UI。
- `git status --short` 只出现本任务预期删除、移动和新增。

### Step 4：提交

```bash
git add -A
git commit -m "chore: establish V2 workspace"
```

---

## Task 2：先用测试固定 Rust IPC 合约

**创建：**

- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/Cargo.lock`
- `apps/desktop/src-tauri/build.rs`
- `apps/desktop/src-tauri/src/domain/mod.rs`
- `apps/desktop/src-tauri/src/domain/contracts.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/src/main.rs`

### Step 1：写失败测试

在 `domain/contracts.rs` 的测试模块先写序列化测试，固定 camelCase JSON 和枚举的 lower camel case 值：

```rust
#[test]
fn bootstrap_contract_serializes_for_frontend() {
    let state = BootstrapState {
        schema_version: 1,
        platform: PlatformId::MacosArm64,
        tools: vec![],
    };

    assert_eq!(
        serde_json::to_value(state).unwrap(),
        serde_json::json!({
            "schemaVersion": 1,
            "platform": "macosArm64",
            "tools": []
        })
    );
}
```

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap_contract_serializes_for_frontend
```

Expected: FAIL，因为 DTO 尚未定义。

### Step 2：实现最小权威 DTO

所有导出 DTO 同时派生 `Serialize`、`Deserialize`、`Clone`、`Debug`、`PartialEq`、`Eq` 和 `TS`，并统一使用：

```rust
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
```

固定类型：

```rust
pub enum PlatformId {
    WindowsX64,
    MacosArm64,
}

pub enum ToolId {
    Winget,
    Git,
    Nodejs,
    ClaudeCode,
    CodexCli,
}

pub enum ToolRequirement {
    Default,
    Optional,
    Unavailable,
}

pub enum ToolCapability {
    Detect,
    Install,
    Upgrade,
    Repair,
}

pub struct PlatformPolicy {
    pub platform: PlatformId,
    pub requirement: ToolRequirement,
}

pub struct ToolDefinition {
    pub id: ToolId,
    pub label_key: String,
    pub platform_policies: Vec<PlatformPolicy>,
    pub capabilities: Vec<ToolCapability>,
    pub runtime_dependencies: Vec<ToolId>,
}

pub struct BootstrapState {
    pub schema_version: u16,
    pub platform: PlatformId,
    pub tools: Vec<ToolDefinition>,
}
```

`Cargo.toml` 的核心依赖精确到兼容小版本：

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
tauri = { version = "2.11", features = [] }
ts-rs = { version = "12.0.1", features = ["serde-compat"] }

[dev-dependencies]
serde_json = "1.0"

[build-dependencies]
tauri-build = { version = "2.5", features = [] }
```

应用 crate 名为 `code-ready-desktop`，library 名为 `code_ready_desktop_lib`，edition 2024，`rust-version = "1.88"`。

### Step 3：验证

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap_contract_serializes_for_frontend
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS，无 warning。

### Step 4：提交

```bash
git add apps/desktop/src-tauri
git commit -m "feat: define V2 IPC contracts"
```

---

## Task 3：用不变量测试驱动内置工具注册表

**创建：**

- `apps/desktop/src-tauri/src/domain/tool_registry.rs`

**修改：**

- `apps/desktop/src-tauri/src/domain/mod.rs`
- `apps/desktop/src-tauri/src/lib.rs`

### Step 1：先写注册表失败测试

测试必须覆盖：

1. 工具 ID 唯一。
2. 每个工具恰好各有一条 Windows 和 macOS policy。
3. 每项 capability 不重复。
4. runtime dependency 不指向自己且引用已知工具。
5. 依赖图无环。
6. 注册表严格等于下表。
7. Claude Code 与 Codex CLI 的 runtime dependency 均为空。
8. 不存在独立 npm `ToolId`。

固定矩阵：

| ToolId | Windows x64 | macOS arm64 | 能力 | 运行时依赖 |
|---|---|---|---|---|
| Winget | Default | Unavailable | Detect, Install, Upgrade, Repair | 无 |
| Git | Default | Default | Detect, Install, Upgrade, Repair | 无 |
| Nodejs | Default | Optional | Detect, Install, Upgrade, Repair | 无 |
| ClaudeCode | Default | Default | Detect, Install, Upgrade, Repair | 无 |
| CodexCli | Default | Default | Detect, Install, Upgrade, Repair | 无 |

`Nodejs` 的 label key 为 `tools.nodejsAndNpm.name`，明确 npm 随 Node.js 提供。其他 label key：

```text
tools.winget.name
tools.git.name
tools.claudeCode.name
tools.codexCli.name
```

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml tool_registry
```

Expected: FAIL，因为 `built_in_tool_registry()` 尚未实现。

### Step 2：最小实现

实现：

```rust
pub fn built_in_tool_registry() -> Vec<ToolDefinition>
```

只返回上表的静态定义，不添加来源 URL、命令、探针或安装配方。不要把未来设计塞进字符串字段。

### Step 3：验证

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml tool_registry
```

Expected: 所有注册表不变量 PASS。

### Step 4：提交

```bash
git add apps/desktop/src-tauri/src/domain
git commit -m "feat: add typed built-in tool registry"
```

---

## Task 4：用 Fake 驱动平台边界和 bootstrap 应用服务

**创建：**

- `apps/desktop/src-tauri/src/platform/mod.rs`
- `apps/desktop/src-tauri/src/platform/fake.rs`
- `apps/desktop/src-tauri/src/platform/native.rs`
- `apps/desktop/src-tauri/src/application/mod.rs`
- `apps/desktop/src-tauri/src/application/bootstrap.rs`

**修改：**

- `apps/desktop/src-tauri/src/lib.rs`

### Step 1：写失败测试

在 `application/bootstrap.rs` 测试：

```rust
#[test]
fn bootstrap_uses_platform_adapter_and_registry() {
    let adapter = Arc::new(FakePlatformAdapter::new(PlatformId::MacosArm64));
    let service = BootstrapService::new(adapter);

    let state = service.get_state();

    assert_eq!(state.schema_version, 1);
    assert_eq!(state.platform, PlatformId::MacosArm64);
    assert_eq!(state.tools, built_in_tool_registry());
}
```

再写 Windows fake 的同类断言，证明业务服务没有编译期平台分支。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap_uses_platform_adapter_and_registry
```

Expected: FAIL，因为 trait、fake 和 service 尚不存在。

### Step 2：实现最小平台边界

接口固定为：

```rust
pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> PlatformId;
}
```

`FakePlatformAdapter` 仅保存传入的 `PlatformId`。

`NativePlatformAdapter` 只能在 `platform/native.rs` 使用 `cfg`：

- `target_os = "windows"` 且 `target_arch = "x86_64"` → `WindowsX64`
- `target_os = "macos"` 且 `target_arch = "aarch64"` → `MacosArm64`
- 其他构建目标返回明确的 unsupported error，不伪装成受支持平台。

`BootstrapService` 持有 `Arc<dyn PlatformAdapter>`，从适配器取平台，从注册表取工具，返回 `BootstrapState`。它不得依赖 Tauri。

### Step 3：增加架构守卫

增加测试读取 `src/domain` 与 `src/application` 的 `.rs` 文件，并断言其中没有：

```text
std::process
powershell
cmd.exe
/bin/sh
cfg(target_os
tauri::
```

该守卫只限制这两个目录；平台目录和未来基础设施目录不受此文本规则约束。

### Step 4：验证并提交

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
git add apps/desktop/src-tauri/src
git commit -m "feat: isolate platform bootstrap boundary"
```

Expected: Rust 测试全部 PASS。

---

## Task 5：生成 TypeScript bindings，并让 CI 能发现漂移

**创建：**

- `apps/desktop/src-tauri/src/bin/export-bindings.rs`
- `apps/desktop/src/shared/api/generated/BootstrapState.ts`
- `apps/desktop/src/shared/api/generated/PlatformId.ts`
- `apps/desktop/src/shared/api/generated/PlatformPolicy.ts`
- `apps/desktop/src/shared/api/generated/ToolCapability.ts`
- `apps/desktop/src/shared/api/generated/ToolDefinition.ts`
- `apps/desktop/src/shared/api/generated/ToolId.ts`
- `apps/desktop/src/shared/api/generated/ToolRequirement.ts`
- `apps/desktop/src/shared/api/generated/index.ts`

### Step 1：写失败的漂移检查

先在 exporter 的测试中验证导出类型集合恰好是：

```text
BootstrapState
PlatformId
PlatformPolicy
ToolCapability
ToolDefinition
ToolId
ToolRequirement
```

并验证生成目录中不存在旧文件。Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml export_bindings
```

Expected: FAIL，因为 exporter 尚不存在。

### Step 2：实现 exporter

`export-bindings` 必须：

1. 使用 `env!("CARGO_MANIFEST_DIR")` 定位 `../src/shared/api/generated`，不依赖调用者 cwd。
2. 只清理该目录内的 `.ts` 生成文件。
3. 对 Task 2 的每个 DTO 调用 `TS::export_all_to`。
4. 生成只含显式类型导出的 `index.ts`，例如
   `export type { BootstrapState } from "./BootstrapState";`，并为其余六种生成类型写出同样的确定性导出行。
5. 输出稳定、可重复；连续运行两次 Git diff 为空。

TypeScript 不得另建手写同名 DTO。

### Step 3：生成并验证漂移

```bash
npm run generate:bindings
git add apps/desktop/src/shared/api/generated
npm run check:bindings
```

Expected:

- 第一次生成产生 8 个 `.ts` 文件。
- 文件加入 index 后，`npm run check:bindings` 退出码 0。

手动破坏一个生成文件，再运行 `npm run check:bindings`，Expected: 非零；随后重新生成恢复。这个红灯验证必须在执行记录中注明。

### Step 4：提交

```bash
git add package.json apps/desktop/src-tauri apps/desktop/src/shared/api/generated
git commit -m "build: generate TypeScript contracts from Rust"
```

---

## Task 6：用前端测试驱动最小可启动 UI

**创建：**

- `apps/desktop/src/main.tsx`
- `apps/desktop/src/app/App.tsx`
- `apps/desktop/src/app/App.test.tsx`
- `apps/desktop/src/app/styles.css`
- `apps/desktop/src/shared/api/client.ts`
- `apps/desktop/src/shared/i18n/zh-CN.ts`

### Step 1：写失败 UI 测试

mock `getBootstrapState()` 返回：

```ts
{
  schemaVersion: 1,
  platform: "macosArm64",
  tools: [
    {
      id: "git",
      labelKey: "tools.git.name",
      platformPolicies: [
        { platform: "windowsX64", requirement: "default" },
        { platform: "macosArm64", requirement: "default" }
      ],
      capabilities: ["detect", "install", "upgrade", "repair"],
      runtimeDependencies: []
    }
  ]
}
```

测试必须断言：

- 初始显示“正在读取当前设备…”
- 完成后显示“Code-Ready V2”
- 显示“macOS Apple Silicon”
- 显示“已载入 1 项内置工具定义”
- 不显示安装、提权、登录、API Key 或遥测控件

Run:

```bash
npm run test --workspace @code-ready/desktop -- App.test.tsx
```

Expected: FAIL，因为 App/client 尚未实现。

### Step 2：实现 API client

`client.ts` 只从 `generated/index.ts` 导入 `BootstrapState`，并调用：

```ts
invoke<BootstrapState>("get_bootstrap_state")
```

不得手写 `PlatformId` 或 `ToolId` union。

### Step 3：实现最小 UI

页面只包含：

- V2 标题和“重写基线”说明；
- 当前支持平台；
- 内置工具定义数量；
- “检测与引导安装将在下一切片接入”的明确占位说明。

所有用户可见文本从 `zh-CN.ts` 的 typed message object 读取。Slice 0 不引入完整 i18n 库，但禁止把文案散落在 JSX。

### Step 4：验证

```bash
npm run test --workspace @code-ready/desktop -- App.test.tsx
npm run typecheck
npm run lint
npm run build
```

Expected: 全部 PASS，Vite 生成 `apps/desktop/dist`。

### Step 5：提交

```bash
git add apps/desktop/src apps/desktop/index.html apps/desktop/*.json apps/desktop/*.js apps/desktop/*.ts
git commit -m "feat: add V2 bootstrap interface"
```

---

## Task 7：接通 Tauri command 与真实 native adapter

**创建：**

- `apps/desktop/src-tauri/src/api/mod.rs`
- `apps/desktop/src-tauri/src/api/commands.rs`

**修改：**

- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src-tauri/src/main.rs`

### Step 1：先写 command 边界测试

把 command 内核写成可直接测试的普通函数，Tauri attribute 只做薄封装。测试用 `FakePlatformAdapter(WindowsX64)` 调用内核并断言返回：

- `schemaVersion = 1`
- `platform = windowsX64`
- 5 个工具定义

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml get_bootstrap_state
```

Expected: FAIL，因为 command 尚不存在。

### Step 2：实现 command

Tauri builder：

1. 启动时构造一次 `NativePlatformAdapter`。
2. 以 managed state 保存 `BootstrapService`。
3. 只注册一个 command：`get_bootstrap_state`。
4. 不注册 dialog、opener、shell、process、http、fs、updater 插件。

`main.rs` 只调用 library 的 `run()`。command 不得接受任意命令、路径、URL 或环境变量。

### Step 3：验证

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml get_bootstrap_state
cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Expected: PASS。

### Step 4：提交

```bash
git add apps/desktop/src-tauri/src
git commit -m "feat: expose bootstrap state through Tauri"
```

---

## Task 8：用自动测试锁定 CSP、capability 和 macOS 13

**创建：**

- `apps/desktop/src-tauri/tauri.conf.json`
- `apps/desktop/src-tauri/capabilities/main.json`
- `apps/desktop/src-tauri/tests/security_configuration.rs`

### Step 1：写失败安全配置测试

测试读取上述 JSON，并断言：

- `identifier` 暂用 `com.codeready.v2`；后续确定正式产品名时再迁移。
- 窗口 label 为 `main`。
- CSP 是非空字符串。
- CSP 包含 `default-src 'self'` 与 `script-src 'self'`。
- CSP 不包含 `unsafe-eval`、`https:`、通配符源或远程脚本。
- `connect-src` 仅允许 Tauri IPC 所需的 `ipc:` 和 `http://ipc.localhost`。
- capability 只绑定 `main` 窗口。
- capability permissions 是精确 allowlist，不包含 shell、process、fs、http、dialog、opener、updater。
- bundle targets 为 `app` 与 `nsis`；Slice 0 不构建 MSI。
- macOS minimum system version 为 `13.0`。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test security_configuration
```

Expected: FAIL，因为配置尚未创建。

### Step 2：实现最小安全配置

CSP 使用：

```text
default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: http://asset.localhost; font-src 'self'; connect-src ipc: http://ipc.localhost
```

`unsafe-inline` 仅允许在 style；script 不允许。capability 只授予最小 core app/event/window 权限；如果 Tauri 编译证明某项不需要，删除而不是保留。

Tauri build 配置：

```text
beforeDevCommand: npm run dev:web
beforeBuildCommand: npm run build:web
devUrl: http://localhost:1420
frontendDist: ../dist
```

### Step 3：验证配置和本机 app 构建

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --test security_configuration
npm run tauri:build
```

Expected: 安全测试 PASS，当前平台产生无签名 app executable；不要求安装包签名或公证。

### Step 4：提交

```bash
git add apps/desktop/src-tauri/tauri.conf.json apps/desktop/src-tauri/capabilities apps/desktop/src-tauri/tests
git commit -m "security: lock down V2 Tauri surface"
```

---

## Task 9：建立双平台 CI，并准确区分编译 CI 与干净机验收

**创建：**

- `.github/workflows/ci.yml`
- `docs/testing/v2-platform-matrix.md`

### Step 1：先写 CI 配置静态测试

在 `security_configuration.rs` 增加对 `.github/workflows/ci.yml` 的静态断言：

- 包含 `windows-2025` 和 `macos-15`。
- 包含 Node 24。
- 包含 `MACOSX_DEPLOYMENT_TARGET: "13.0"`。
- 包含 `npm ci`、`npm run check`、`npm run tauri:build`。
- 不把 `windows-2025` 描述为 Windows 11。

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml ci_matrix
```

Expected: FAIL，因为 workflow 尚不存在。

### Step 2：创建 workflow

单一 matrix job：

```yaml
strategy:
  fail-fast: false
  matrix:
    os: [windows-2025, macos-15]
runs-on: ${{ matrix.os }}
env:
  MACOSX_DEPLOYMENT_TARGET: "13.0"
```

步骤顺序：

1. checkout。
2. setup-node 24，cache npm。
3. 安装 stable Rust（含 `rustfmt`、`clippy`）。
4. 打印并断言架构：
   - Windows 输出必须是 AMD64/x86_64。
   - macOS 输出必须是 arm64/aarch64。
5. `npm ci`。
6. `npm run check`。
7. `npm run tauri:build`。

不要上传公开发布制品，不签名，不公证。

### Step 3：记录平台门禁语义

`docs/testing/v2-platform-matrix.md` 必须明确：

- `windows-2025` 是 GitHub 托管的 Windows Server x64 编译/测试环境，不等于 Windows 11。
- Slice 0 的 Windows 11 x64 结论只能写“待后续干净 VM/机器验收”，不能写已通过。
- `macos-15` job 必须实测为 arm64；若 runner label 不再提供 arm64，CI 要红灯而不是降级到 Intel。
- `MACOSX_DEPLOYMENT_TARGET=13.0` 只建立编译兼容目标，不等于已经在 macOS 13 干净机验收。
- 公开发布仍需 Windows 签名、macOS Developer ID 签名/公证/staple；Slice 0 不声称完成。

### Step 4：本地验证并提交

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml ci_matrix
git add .github/workflows/ci.yml docs/testing/v2-platform-matrix.md apps/desktop/src-tauri/tests
git commit -m "ci: verify V2 on Windows and Apple Silicon"
```

Expected: 静态 CI 测试 PASS。远端 GitHub Actions 的真实结果在 push/PR 后确认。

---

## Task 10：根级验收、文档对齐和 Slice 0 收口

**修改：**

- `README.md`
- `.gitignore`
- 仅在实际命令需要时修改前述配置文件

### Step 1：更新 README

README 首屏必须说明：

- 当前分支是内部代号 Code-Ready V2 的 clean-slate rewrite。
- V1 仅在 Git 历史/main 中保留，不再参与当前构建。
- 当前 Slice 0 只有可启动骨架与契约，不具备工具安装能力。
- 支持目标是 Windows 11 x64 与 macOS 13+ Apple Silicon。
- macOS 不默认依赖 Node/npm；Windows 的 Node.js/npm 仍是默认工具，但 Claude/Codex 都不依赖它。
- 仓库当前没有开源许可证。

不要保留已经失效的 V1 截图作为 V2 当前界面。

### Step 2：运行完整验证

先清理构建输出，不删除依赖锁文件：

```bash
git status --short
npm ci
npm run check
npm run build
npm run tauri:build
git diff --check
git status --short
```

Expected:

- 所有 typecheck、lint、Vitest、fmt、clippy、Rust tests、binding drift checks PASS。
- Vite build PASS。
- 当前平台 Tauri 无签名 app build PASS。
- `git diff --check` 无输出。
- status 只含 README/.gitignore 的预期变更；没有 `dist`、`target`、日志、缓存或临时生成物。

### Step 3：人工约束审查

Run:

```bash
rg -n "plugin-(dialog|opener|shell|process|http|fs)|unsafe-eval|csp[\"']?\\s*:\\s*null|powershell|cmd\\.exe|/bin/sh|curl.+\\|.+sh|irm.+\\|.+iex" apps package.json
rg -n "Nodejs|nodejs|npm" apps/desktop/src-tauri/src/domain apps/desktop/src/shared/api/generated
```

Expected:

- 第一条无输出，CSP 测试文件中的否定 fixture 除外；若测试文字命中，人工确认生产配置/代码无命中。
- 第二条只在 Node.js/npm 自身工具定义中出现；Claude/Codex dependency 为空。

检查：

```bash
git grep -n "from .*types/\\|type PlatformId\\|type ToolId" -- apps/desktop/src ':!apps/desktop/src/shared/api/generated'
```

Expected: 不存在手写协议 union。

### Step 4：请求代码审查

使用 `requesting-code-review` 对 Slice 0 的需求、设计规范和 diff 做一次独立审查。必须处理 P0/P1 问题；低优先级问题记录后才能收口。

### Step 5：最终提交

```bash
git add README.md .gitignore
git commit -m "docs: document V2 rebuild baseline"
git status --short --branch
```

Expected: 工作树干净，分支仍基于 `codex/v2-rebuild` 的 Slice 0 执行分支。

---

## Slice 0 完成定义

只有同时满足以下条件才能声称 Slice 0 完成：

1. 所有 10 个任务均有对应提交或有书面理由合并提交。
2. `npm run check`、`npm run build`、本机 `npm run tauri:build` 有本轮新鲜成功输出。
3. 生成绑定的红灯漂移测试已实际验证。
4. Rust 注册表测试证明双平台 policy、依赖无环、Claude/Codex 不依赖 Node。
5. security configuration test 证明 CSP 和 capabilities 收敛。
6. GitHub Actions 的 Windows x64 与 macOS arm64 jobs 已真实通过；若尚未 push，只能报告“本地完成，远端双平台验收待运行”，不得把 Slice 0 标为最终通过。
7. Windows 11 x64 和 macOS 13 干净机仍明确标记为后续 acceptance gate，未被 CI runner 冒充。
8. 工作树干净，没有 V1 生产代码、生成漂移或构建垃圾。

## 明确推迟到后续 Slice

- Slice 1：真实工具检测、事实状态、首次向导、状态中心。
- Slice 2：配置、代理、日志、诊断导出、V1 配置导入预览。
- Slice 3：计划摘要、确认、串行任务、取消、journal、恢复。
- Slice 4/5：Claude Code、Codex CLI 原生安装。
- Slice 6：窄权限 helper 和一次授权。
- Slice 7/8：Windows 与 macOS 系统工具真实安装。
- Slice 9：国内镜像、应用更新、完整恢复。
- Slice 10：签名、公证、SBOM、公开发布与干净机器验收。
