# Code-Ready V2

Code-Ready V2 是内部代号下的 clean-slate rewrite，目标平台为 Windows 11 x64 与 macOS 13+ Apple Silicon。

## 当前状态

Slice 0 只提供可启动的桌面骨架、Rust 权威 IPC 契约、类型化内置工具注册表、平台抽象和双平台编译 CI。它暂不检测、下载、安装或修复任何工具。

V1 只保留在 Git 历史和 `main` 分支中，不参与当前 workspace 的构建。仓库当前没有开源许可证，也没有提交 `LICENSE` 文件。

当前应用显示平台和内置工具定义摘要。真实工具检测、安装计划、权限、配置、日志、迁移和更新能力按后续切片接入。

## 平台与工具边界

- Windows 默认工具矩阵包含 WinGet、Git、Node.js/npm、Claude Code 和 Codex CLI。
- macOS 默认工具矩阵包含 Git、Claude Code 和 Codex CLI；Node.js/npm 只作为可选开发环境显示，不默认加入计划。
- npm 是 Node.js 的派生能力，不是独立工具。
- Claude Code 与 Codex CLI 的运行时安装策略不依赖 Node.js/npm。

CI runner 只证明编译与自动化检查：`windows-2025` 是 Windows Server x64 环境，`macos-15` 必须实际为 arm64。Windows 11 和 macOS 13+ Apple Silicon 的干净机验收仍是后续门禁。

## 开发

应用位于 `apps/desktop`，仓库根目录是 npm workspace。需要 Node.js 24、npm 11 和 Rust stable（最低 Rust 1.88）。

```bash
npm install
npm run check
npm run build
npm run tauri:build
```

`npm run tauri:build` 在 Slice 0 生成当前平台的无签名应用可执行文件，不生成公开安装包，不代表签名、公证或干净机验收已完成。

## 后续切片

后续切片将按设计逐步加入只读检测、配置与日志、安装计划和可恢复安装会话，再接入真实工具与窄权限执行器。Slice 0 不启用遥测，也不包含安装、下载、提权、日志上传、代理接管或应用更新能力。
