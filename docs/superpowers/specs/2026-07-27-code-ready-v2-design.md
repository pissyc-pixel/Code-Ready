# Code-Ready V2 从零重构设计

**日期：** 2026-07-27

**状态：** 待书面规范复核

**规划模型：** GPT-5.6 Sol，推理强度 High

**执行模型：** GPT-5.6 Luna，推理强度 Max

## 1. 背景

现有 `main` 分支是一个 Windows-only 的 Tauri 2 + React 应用，已经实现环境检测、安装、配置和日志诊断，但当前前端无法构建：`src/pages/LogsPage.tsx` 引用了仓库中不存在的 `src/components/logs/LogViewer.tsx`。现有前端状态和动作集中在大型 `App.tsx`，样式集中在大型 `App.css`；Rust 侧已有较多行为测试，但模块边界仍受历史迭代影响。

V2 不在现有实现上继续拆补，而是在 `codex/v2-rebuild` 分支建立新的架构。旧代码、旧测试和文档只用于核对产品行为，不直接复制生产实现。

## 2. 产品目标

Code-Ready V2 面向不熟悉命令行的用户，让用户在 Windows 或 macOS 上完成以下闭环：

1. 自动识别当前开发环境。
2. 用非技术语言解释缺失、过旧或异常的工具。
3. 让用户逐项确认安装计划。
4. 串行执行已确认的安装、升级或修复。
5. 在失败、取消或应用重启后提供明确恢复路径。
6. 通过本地日志和脱敏诊断包帮助排查问题。

V2 同时发布 Windows 和 macOS 版本，并保持相同的产品流程、状态语义和主要能力。

## 3. 非目标

V2 首版明确不包含：

- 一键静默安装所有工具；
- 工具卸载；
- 离线安装包；
- 外部插件系统；
- Claude、Codex 登录或 API Key 管理；
- 系统代理接管；
- 默认遥测或崩溃上传；
- Intel Mac、Windows 10、Windows ARM64；
- 自动执行无法验证来源或完整性的安装包；
- 删除或改写 V1 配置；
- 对外授予开源许可证。

## 4. 平台和工具范围

### 4.1 支持平台

| 平台 | 首版范围 |
|---|---|
| Windows | Windows 11 x64 |
| macOS | macOS 13.0 及以上，Apple Silicon |

macOS 13.0 是由首版工具中最高的官方系统要求决定的：Claude Code 当前原生版本要求 macOS 13.0 或更高版本。

### 4.2 Windows 工具

Windows 首次向导默认检测并推荐：

- Winget；
- Git for Windows；
- Node.js LTS 和 npm；
- Claude Code；
- Codex CLI。

Node.js/npm 是 Windows 新手开发环境的默认组成，但不是 Claude Code 或 Codex CLI 的安装前置。

### 4.3 macOS 工具

macOS 首次向导默认检测并推荐：

- Git；
- Claude Code；
- Codex CLI。

Node.js/npm 在 macOS 上只作为可选开发环境显示，不默认加入安装计划，也不阻塞 Claude Code 或 Codex CLI。

## 5. 技术方案

### 5.1 技术栈

- Tauri 2：桌面壳层、命令桥接、打包与更新；
- React + TypeScript：共享 UI、状态展示和用户交互；
- Rust：领域模型、用例、工具适配器、平台适配器、持久化和安全边界；
- 版本化原子文件：Slice 2 的单文档设置、迁移记录和最近有效备份；
- SQLite：仅在后续安装会话 journal、并发状态或关系查询证明原子文件不足时再引入；
- 滚动日志文件：本地诊断日志，保留 30 天。

Tauri 窗口使用严格 CSP，禁止远程脚本、远程代码注入和未声明连接目标。公开构建与内部构建都不得把 CSP 设置为 `null`。

### 5.2 仓库结构

```text
apps/
  desktop/
    src/
      app/
      features/
        onboarding/
        dashboard/
        install-session/
        diagnostics/
        settings/
      shared/
        api/
        i18n/
        ui/
    src-tauri/
      src/
        domain/
        application/
        commands/
        infrastructure/
        platform/
          windows/
          macos/
        tools/
          winget/
          git/
          node/
          claude/
          codex/
```

文件按业务职责组织，而不是把所有 hooks、组件或命令堆入全局目录。每个工具模块只描述该工具自身的检测、规划、执行和验证；平台模块只描述操作系统能力。

## 6. 架构边界

### 6.1 Rust 是事实来源

Rust Core 是以下状态的唯一事实来源：

- 工具检测结果；
- 安装计划；
- 安装会话状态；
- 下载来源和完整性校验结果；
- 配置和迁移状态；
- 日志保留和诊断导出状态。

React 不拼装 shell 命令，不自行推断安装是否成功，也不把仅存在于组件内存中的状态当作任务事实。

### 6.2 共享领域模型

核心类型至少包括：

```text
Platform
ToolId
ToolDefinition
ToolRequirementLevel
ToolCapability
ObservedToolState
VersionStatus
DetectionEvidence
SourceDefinition
InstallPlan
InstallStep
InstallSession
InstallEvent
AppSnapshot
AppError
```

Rust `serde` 结构体是 IPC 合约的权威定义。构建流程从 Rust 类型生成 TypeScript 类型，CI 检查生成文件无漂移；前端不得维护第二套手写同名协议。

工具事实状态和操作状态必须分离。`ObservedToolState` 只能描述机器当前事实，`InstallSession` 和 `InstallStep` 才描述计划、下载、执行、失败、取消和恢复。取消安装不能把工具事实状态改写为 `missing`。

### 6.3 工具注册表

工具通过内置、类型化注册表接入：

```text
ToolDefinition
  identity
  platformAvailability
  requirementLevel
  capabilities
  detector
  sourcePolicy
  planner
  executor
  verifier
```

首版不从磁盘或网络加载第三方工具定义。新增工具必须经过代码审查、签名发布和完整测试。

### 6.4 平台适配器

共享用例只依赖以下平台能力接口：

```text
ProcessRunner
PathEnvironment
PrivilegeBroker
DownloadClient
IntegrityVerifier
PackageManager
FileSystem
Clock
SystemInfo
```

Windows 和 macOS 各自实现这些接口。工具模块不能直接调用 PowerShell、注册表、`xcode-select` 或 macOS 授权 API。

## 7. 前后端命令和事件

### 7.1 命令

前端只通过窄命令接口表达用户意图：

```text
bootstrap() -> AppSnapshot
detect_tools(tool_ids?) -> DetectionRunId
create_install_plan(selection) -> InstallPlan
approve_install_plan(plan_id, plan_digest) -> InstallSession
cancel_install_session(session_id) -> InstallSession
update_settings(patch) -> Settings
import_legacy_config(selection) -> MigrationResult
export_diagnostics(destination) -> DiagnosticsResult
check_app_update() -> AppUpdateStatus
```

`approve_install_plan` 必须携带计划摘要。若用户确认后计划内容发生变化，后端拒绝执行并要求重新确认。

### 7.2 事件

长时间操作通过事件流上报：

```text
detection.changed
install.session.changed
install.step.changed
install.log.appended
settings.changed
app.update.changed
```

事件只用于及时更新 UI。前端启动、重新连接或发现序号缺口时必须调用 `bootstrap()` 获取完整快照，不能依赖事件重放恢复事实。

每个事件包含：

- 单调递增序号；
- 会话或检测运行 ID；
- 时间戳；
- 稳定事件类型；
- 可本地化的错误代码；
- 已脱敏的展示数据。

## 8. 安装状态机

单个安装会话使用以下状态：

```text
Draft
  -> AwaitingApproval
  -> Authorizing
  -> Running
  -> Verifying
  -> Completed

Authorizing/Running/Verifying
  -> Cancelling
  -> Cancelled

Authorizing/Running/Verifying
  -> Failed

应用异常退出
  -> Interrupted
  -> Reconcile
  -> Completed | PartialFailure | Failed | Cancelled | NeedsUserAction
```

约束：

- 同一时间最多存在一个活动安装会话；
- 工具和步骤串行执行；
- 取消只阻止尚未开始的步骤，并尽力终止当前子进程；
- 单个工具失败后默认暂停队列，用户可以重试、跳过或结束；应用不能静默继续；
- 不承诺回滚 Winget、Apple 系统安装器或其他外部安装器已经完成的更改；
- 每个步骤开始和结束时写入持久化日志和任务快照；
- `Interrupted` 会话重启后先重新检测受影响工具，不自动重放外部命令；
- 只有验证器确认目标状态后，步骤才能标记为成功。

## 9. 权限模型

### 9.1 共同原则

- 主应用始终以普通用户权限运行；
- 检测、配置、日志和诊断导出不要求提权；
- 用户确认完整安装计划后，Code-Ready 每个安装会话最多触发一次应用控制的授权；
- 用户拒绝授权后当前会话终止；再次尝试必须创建并重新确认新会话；
- 高权限边界只接受类型化、白名单化步骤，不接受任意命令字符串；
- 授权会话结束后，高权限执行器立即退出。

### 9.2 Windows

Windows 使用独立、签名的安装执行器：

- 通过 UAC 在安装会话开始时提权；
- 使用带随机 nonce 的本地 IPC；
- 校验父应用身份、安装计划摘要和步骤枚举；
- 只执行内置工具适配器允许的固定程序和参数；
- 不接受来自 React、URL、日志或配置文件的可执行命令。

公开构建使用正式 Authenticode 签名；权限链路的内部集成构建必须使用开发签名。完全未签名的内部构建可以验证普通权限流程，但不得宣称已验证高权限执行器。

### 9.3 macOS

Claude Code 和 Codex CLI 的原生安装目标位于用户目录时不申请管理员权限。Git 的 Apple Command Line Tools 安装和其他系统级动作交给 Apple 系统界面或受限授权执行器。

Code-Ready 自己控制的授权每个会话最多一次；Apple Installer、Command Line Tools 许可协议等操作系统拥有的可信弹窗不计入该承诺，并在计划确认页提前说明。macOS 权限链路的内部集成构建使用与主应用相同 Team ID 的开发签名。

## 10. 工具行为

### 10.1 检测结果

每个工具返回以下事实状态之一：

```text
unknown
absent
present_healthy
present_path_issue
present_broken
unsupported
```

版本另行返回：

```text
current
outdated
newer_than_known
not_comparable
unknown
```

`checking`、`downloading`、`executing`、`failed`、`cancelled` 等只属于检测运行或安装任务，不属于工具事实状态。

结果必须附带可验证证据，例如解析后的版本、可执行文件路径、命令退出码类别或包管理器记录。原始 stdout/stderr 只写入脱敏日志，不直接决定 UI 文案。

### 10.2 能力矩阵

| 平台和工具 | 检测 | 安装 | 升级 | 修复 |
|---|---:|---:|---:|---:|
| Windows Winget | 是 | 系统引导 | 系统引导 | App Installer 修复引导 |
| Windows Git | 是 | 是 | 是 | 是 |
| Windows Node.js/npm | 是 | 是 | 是 | 是 |
| Windows Claude Code | 是 | 是 | 是 | 是 |
| Windows Codex CLI | 是 | 是 | 是 | 是 |
| macOS Git | 是 | Apple CLT | 系统更新 | Apple CLT 修复引导 |
| macOS Node.js/npm（可选） | 是 | 是 | 是 | 是 |
| macOS Claude Code | 是 | 是 | 是 | 是 |
| macOS Codex CLI | 是 | 是 | 是 | 是 |

npm 是 Node.js 工具定义中的派生能力，不作为独立安装任务。某个动作只能由对应工具适配器明确声明；不支持的动作显示原因，禁止用“卸载后重装”伪装成修复。

### 10.3 安装来源

| 工具 | 官方默认 | 可验证回退 |
|---|---|---|
| Winget | Windows App Installer / 系统 Winget | 无可信镜像时仅提供修复引导 |
| Git (Windows) | Winget `Git.Git` | Git for Windows 官方发布源 |
| Git (macOS) | Apple Command Line Tools | Apple 系统更新 |
| Node.js (Windows) | Winget `OpenJS.NodeJS.LTS` / Node 官方发布 | npmmirror Node 二进制镜像，必须对照 Node 官方校验值 |
| Node.js (macOS，可选) | Node 官方 Apple Silicon 发布 | npmmirror Node 二进制镜像，必须对照 Node 官方校验值 |
| Claude Code | Anthropic 原生安装和签名清单 | 没有可验证国内镜像时不提供镜像；允许用户代理 |
| Codex CLI | OpenAI 原生安装，`releases.openai.com` | OpenAI 安装器内置的 GitHub Releases 官方回退 |

来源策略：

1. 默认只使用官方源；
2. 官方源失败后解释原因；
3. 用户显式选择后才使用国内镜像；
4. 镜像域名必须存在于应用内置白名单；
5. 下载内容必须通过官方 SHA-256、签名清单或平台代码签名校验；
6. 无法获得独立官方校验值时拒绝执行；
7. 镜像失败不自动切换到其他未知域名；
8. 日志记录来源 ID、版本、校验算法和结果，但不记录代理凭据。

安装器禁止使用 `irm ... | iex`、`curl ... | sh` 等边下载边执行形式。上游只提供脚本时，Code-Ready 先将脚本或制品下载到隔离缓存，限制重定向和大小，验证已批准的哈希、签名或发布者身份，再执行本地副本。高权限执行器只能消费已经验证的本地制品描述，不能自行选择网络来源。

### 10.4 版本策略

- Git：检测当前版本并按发布清单中的最低支持版本判断；
- Node.js：默认推荐当前 LTS 主版本，不自动切换用户已有的版本管理器；
- Claude Code：默认稳定通道；
- Codex CLI：使用 OpenAI 原生安装器的当前非预发布版本；
- 版本下限和来源元数据随签名应用版本发布，不从未签名远程配置动态下发；
- 已安装版本低于发布清单下限时显示升级建议，用户确认后才升级。

## 11. 用户流程

### 11.1 首次向导

```text
欢迎与隐私承诺
-> 平台支持检查
-> 网络和代理检查
-> 自动检测
-> 解释环境状态
-> 用户选择工具
-> 展示来源、权限和安装计划
-> 用户确认
-> 安装与验证
-> 完成摘要
-> 状态中心
```

向导不使用“修复一切”“智能优化”等模糊承诺。每个操作说明将修改什么、使用什么来源、是否需要授权，以及失败后如何恢复。

### 11.2 状态中心

状态中心按用户任务组织：

- 需要处理；
- 正常可用；
- 可选增强；
- 最近安装；
- 诊断和日志。

不把包管理器输出或内部状态码直接暴露给新手。高级详情可以展开查看。

### 11.3 错误恢复

`AppError` 使用稳定分类：

```text
network
permission
integrity
source_unavailable
version_conflict
path_conflict
external_installer
cancelled
unsupported
internal
```

每个错误包含：

- 对用户有意义的标题；
- 已发生的事实；
- 未完成的动作；
- 是否安全重试；
- 推荐恢复动作；
- 可复制的诊断 ID。

完整技术细节进入脱敏日志。

## 12. 配置、日志和迁移

### 12.1 配置

Slice 2 使用严格 schema 的版本化 JSON 原子文件保存：

- 设置文件 schema 版本与迁移来源；
- 语言；
- 网络模式和不含凭据的 HTTP/HTTPS 代理设置；
- npm registry 选择；
- 镜像回退偏好；
- 向导完成状态；
- 工具选择；
- 安装会话快照；
- 旧配置迁移记录。

代理密码、API Key、Claude/Codex 登录凭据不进入 Code-Ready 数据库。

首版不支持 SOCKS5，也不接受任意 npm registry URL。npm registry 只能选择官方 registry 或应用内置白名单项。系统代理表示读取操作系统代理设置；手动代理仅作用于 Code-Ready 支持的下载器和明确支持代理环境变量的安装步骤。Winget 等不接受应用级代理注入的外部工具继续使用其系统网络策略，并在计划页说明。

设置文件只允许应用拥有的固定路径和固定文件名，不接受前端路径。写入采用同目录私有临时文件、完整写入与同步、原子替换、父目录同步，并保留最近有效备份；结构迁移必须可重复执行，迁移失败时回到可启动的只读恢复状态。路径、符号链接或权限检查失败时不得降级到不安全写入。只有后续安装会话 journal、并发状态或关系查询证明这一模型不足时，才允许通过单独设计审查引入 SQLite。

### 12.2 日志

- 本地日志默认保留 30 天；
- 按日期和大小滚动；
- 所有日志总容量上限为 100 MiB，达到上限时先清理最旧日志；
- 写入前执行脱敏；
- 诊断导出再次执行脱敏；
- 导出前展示文件清单；
- token、Authorization、cookie、URL 查询参数和常见 API Key 格式必须被遮蔽；
- 导出时用户主目录替换为稳定占位符；
- 零遥测，日志不会自动上传。

日志采用结构化字段白名单，默认不记录完整环境、完整命令行或任意 HTTP 响应体。正则脱敏是第二道防线，不是允许先记录敏感内容的理由。

### 12.3 V1 配置迁移

V2 首次运行时只读检测 Windows V1 配置 `%APPDATA%\ai-coding-installer\config.json`：

- 展示可导入字段；
- 默认不勾选代理 URL；
- 不导入任何疑似凭据；
- 只允许导入代理模式和仍在白名单中的 npm registry 选择；
- 明确忽略 ccSwitch、订阅页、OpenCode、Python 和自定义下载源；
- 所有工具安装状态均重新检测，不能从 V1 继承；
- 用户确认后写入 V2 数据库；
- 记录迁移来源和结果；
- 不修改或删除 V1 文件；
- 同一 V1 配置不会自动重复导入。

## 13. 国际化和可访问性

- 首版默认简体中文；
- 所有用户文案使用稳定 key，不散落硬编码在业务组件；
- 错误代码与文案分离；
- 日期、路径、数字和平台名称通过格式化层展示；
- UI 从首版起支持键盘操作、焦点可见、语义标签和系统缩放；
- 英文资源可以在后续版本加入而无需修改业务逻辑。

## 14. 应用更新和分发

- 应用检查更新后向用户展示版本、签名状态和发行说明；
- 应用可以在后台检查已签名的更新元数据，但不得在用户确认前下载或安装更新包；
- 用户确认后才下载或安装；
- 内部测试允许未签名构建，但必须醒目标注且不得作为公开下载；
- Windows 公开版本必须使用代码签名证书；
- macOS 公开版本必须使用 Developer ID 签名并完成 Apple 公证；
- 更新包必须由发布密钥签名；
- 公开发布不得依赖开发机本地路径或个人证书配置；
- V2 使用稳定内部应用标识 `com.pissycpixel.codeready`，产品展示名称可以在公开发布前调整；
- Windows Publisher 和 Apple Team ID 从签名证书及发布密钥中读取，不在源码中伪造；
- 产品正式名称在公开发布前确定，开发期间沿用 `Code-Ready V2` 内部代号。

## 15. 测试策略

### 15.1 Rust

- 领域模型和状态机单元测试；
- 安装计划摘要和重确认测试；
- 每个工具检测器的输出 fixture 测试；
- 每个工具规划器的来源和参数测试；
- 下载完整性和签名验证测试；
- 日志脱敏属性测试；
- 崩溃恢复和任务协调测试；
- Windows/macOS 平台适配器契约测试。

### 15.2 React

- 首次向导组件测试；
- 状态中心状态组合测试；
- 安装确认和取消交互测试；
- 错误恢复测试；
- i18n key 完整性测试；
- 键盘和可访问性测试。

### 15.3 集成

- 使用 fake platform adapter 运行完整检测和安装会话；
- Tauri 命令/事件 IPC 契约测试；
- 安装中退出并重新启动的恢复测试；
- 官方源失败、镜像成功的回退测试；
- 哈希不匹配时拒绝执行的测试；
- 高权限执行器拒绝未知步骤、过期 nonce 和摘要不匹配的测试。

### 15.4 CI 和发布门禁

每个合并请求必须通过：

- Windows 11 x64 CI；
- macOS Apple Silicon CI；
- TypeScript 类型检查和 lint；
- React 测试和生产构建；
- Rust `fmt`、`clippy -D warnings`、单元和集成测试；
- 生成 IPC 类型无漂移检查；
- 依赖和许可证清单生成；
- SBOM 和构建制品 SHA-256 生成；
- 安装源白名单和校验元数据检查；
- Tauri capability 最小权限和严格 CSP 检查；
- 无意外外联端点检查。

公开发布前还必须完成：

- Windows 11 干净虚拟机全流程；
- macOS 13+ Apple Silicon 干净机器全流程；
- 国内受限网络回退验证；
- 代理环境验证；
- 取消、失败、重启恢复验证；
- 安装包签名和 macOS 公证验证；
- 日志与诊断包人工脱敏抽查。

## 16. 验收标准

V2 首版只有同时满足以下条件才算完成：

1. Windows 11 x64 和 macOS 13+ Apple Silicon 同时提供可安装构建。
2. 两个平台使用相同的向导、状态语义和错误分类。
3. Windows 能检测和引导安装 Winget、Git、Node.js/npm、Claude Code、Codex CLI。
4. macOS 能检测和引导安装 Git、Claude Code、Codex CLI，并把 Node.js/npm 作为可选项。
5. Claude Code 和 Codex CLI 的原生安装不依赖 Node.js/npm。
6. 用户能在执行前看到并确认完整安装计划。
7. 主应用不以管理员或 root 权限运行。
8. 安装会话串行执行，支持取消、失败和中断恢复。
9. 无法验证完整性的下载绝不执行。
10. 官方源失败时只提供经过白名单和校验的显式回退。
11. 应用不读取、保存或上传 Claude/Codex 登录凭据和 API Key。
12. 默认零遥测，日志保留 30 天并能导出脱敏诊断包。
13. V1 配置只能在用户确认后选择性导入，旧文件保持不变。
14. 所有自动化门禁和双平台干净机器验收通过。
15. 公开构建已完成 Windows 签名或 macOS 签名与公证。
16. 零遥测网络观察未发现分析、设备标识、崩溃或日志上传请求。

## 17. 垂直切片顺序

### Slice 0：重写基线与契约

- 创建独立 V2 应用骨架；
- 建立生成式 IPC 类型；
- 建立 Windows/macOS CI；
- 建立 fake platform adapter；
- 建立类型化注册表、严格 CSP 和平台抽象；
- 产出能启动、能测试、无业务功能的双平台应用。

### Slice 1：首次向导与只读状态中心

- 实现共享工具注册表；
- 实现 Git、Claude Code、Codex CLI 的双平台检测；
- 实现首次向导的检测和解释页面；
- 实现快照和事件序号恢复；
- 使用 i18n key 完成简体中文界面；
- 在普通用户权限下完成两个平台的只读端到端测试。

### Slice 2：配置、网络、日志和旧配置预览

- 实现版本化设置 schema、原子迁移和备份；
- 实现系统代理、无凭据 HTTP/HTTPS 手动代理和白名单 npm registry；
- 实现 30 天、100 MiB 上限的脱敏日志；
- 实现诊断导出和 V1 导入预览；
- 在没有真实安装动作的情况下证明数据安全和零遥测。

### Slice 3：计划、解释与串行状态机

- 实现安装计划、摘要确认和状态机；
- 实现串行任务、取消、日志和崩溃恢复；
- 实现失败暂停、重试、跳过和结束；
- 用 fake installer 完成双平台端到端闭环。

### Slice 4：第一个真实跨平台 AI 工具

- 先接入具有签名清单和逐平台 SHA-256 的 Claude Code；
- 完成双平台检测、安装、升级、修复和复检；
- 使用原生制品，不依赖 Node.js/npm；
- 在普通用户干净机器上完成端到端验收。

### Slice 5：第二个真实跨平台 AI 工具

- 使用相同注册表、下载、校验和执行边界接入 Codex CLI；
- 证明架构没有针对第一个工具写特例；
- 完成双平台干净机器验收。

### Slice 6：窄权限执行器

- 先以受控测试动作实现 Windows/macOS 权限执行器；
- 实现一次授权、IPC 身份校验、计划摘要、nonce 过期和越权拒绝；
- 实现取消、进程树终止和中断恢复；
- 权限集成测试使用开发签名。

### Slice 7：Windows 系统工具

- Windows：Winget、Git、Node.js/npm；
- 完成依赖排序、Winget 缺失/损坏引导；
- Node.js/npm 仍与 Claude Code、Codex CLI 解耦；
- 完成检测、安装、升级、修复和验证。

### Slice 8：macOS 系统工具

- 接入 Apple Command Line Tools Git；
- 接入默认不选的可选 Node.js/npm；
- 提前说明 Apple 系统拥有的安装和许可弹窗；
- 完成检测、安装、升级、修复和验证。

### Slice 9：镜像回退、更新和完整恢复

- 官方源与显式镜像回退；
- 校验、签名和来源审计；
- 应用更新检查和用户确认；
- 跨版本数据库和会话 journal 迁移；
- 官方源不可用、校验失败和代理异常的完整恢复体验。

### Slice 10：发布硬化

- 双平台干净机器验证；
- Windows 签名、发布者和 SmartScreen 验证；
- macOS 签名、公证、staple 和 Gatekeeper 验证；
- SBOM、制品哈希和签名更新清单；
- 国内网络和代理验证；
- 公开发布演练、发布说明和安装包。

每个切片都必须同时在 Windows 和 macOS 上形成可运行、可测试的纵向功能，禁止先完成整套 Windows 实现再复制到 macOS。排序原则是先证明契约、事实状态和可恢复状态机，再连接真实安装器；先打通普通权限原生工具，再引入高风险提权链路和平台特有工具。

## 18. 风险和应对

| 风险 | 应对 |
|---|---|
| 国内没有可信 Claude/Codex 镜像 | 不降低完整性要求；使用官方源、官方回退或用户代理 |
| Windows UAC 执行器扩大攻击面 | 独立签名进程、固定步骤枚举、计划摘要、随机 nonce、会话结束即退出 |
| macOS 系统安装可能出现额外系统弹窗 | 在计划页提前解释；区分 Code-Ready 授权与系统拥有的安装/许可弹窗 |
| 工具安装器输出不稳定 | 以安装后检测器为成功依据，不以输出文本为事实 |
| V1 配置格式损坏 | 只读解析、字段级验证、失败不影响 V2 启动 |
| 同时首发增加平台回归风险 | 每个垂直切片双平台完成，CI 和干净机器验收同时作为门禁 |
| 签名证书尚未准备 | 允许内部未签名测试；公开发布门禁保持关闭 |
| 镜像内容滞后 | 镜像仅显式回退；版本和校验值来自官方元数据 |

## 19. 规划和执行交接

设计和实施计划由 GPT-5.6 Sol（High）完成并提交审查。书面规范通过后，使用 `writing-plans` 生成逐任务 TDD 实施计划。

实现由 GPT-5.6 Luna（Max）在独立执行任务中完成。执行任务必须：

- 从已批准规范和实施计划开始；
- 使用 `codex/v2-rebuild` 的隔离工作树；
- 按垂直切片和 TDD 步骤执行；
- 每个任务完成后运行对应测试；
- 不擅自扩大平台、工具或账号管理范围；
- 在每个切片结束时提交可审查状态；
- 完成前执行逐项验收审计。

## 20. 参考资料

- [OpenAI Codex CLI 安装说明](https://github.com/openai/codex/blob/main/docs/install.md)
- [OpenAI Codex 原生安装器](https://github.com/openai/codex/blob/main/scripts/install/install.sh)
- [Anthropic Claude Code 安装说明](https://code.claude.com/docs/en/setup)
- [Anthropic Claude Code 安装故障排查与 macOS 要求](https://code.claude.com/docs/en/troubleshoot-install)
- [Anthropic Claude Code 二进制完整性说明](https://code.claude.com/docs/en/installation)
- [Apple Command Line Tools 安装说明](https://developer.apple.com/documentation/xcode/installing-the-command-line-tools/)
- [npmmirror 二进制镜像配置](https://www.npmjs.com/package/binary-mirror-config)
