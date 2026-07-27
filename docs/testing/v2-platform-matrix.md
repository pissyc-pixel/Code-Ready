# Code-Ready V2 Slice 0 平台矩阵

Slice 0 的 GitHub Actions 只建立跨平台编译和自动化检查门禁，不替代真实操作系统的干净机验收。

| runner | 本切片用途 | 允许得出的结论 |
| --- | --- | --- |
| `windows-2025` | GitHub 托管的 Windows Server x64 编译/测试环境 | 可以证明 Windows x64 构建与检查在该 runner 上通过；不等于 Windows 11 |
| `macos-15` | 必须由 workflow 的架构断言确认 `arm64` | 可以证明 Apple Silicon runner 上的编译与检查；不等于 macOS 13 干净机 |

CI 矩阵固定使用 Node.js 24、stable Rust、`MACOSX_DEPLOYMENT_TARGET=13.0`，依次运行 `npm ci`、`npm run check` 和无签名的 `npm run tauri:build`。它不上传公开发布制品、不签名、不公证。

`windows-2025` 是 Windows Server 2025 x64 编译环境，Slice 0 不写“Windows 11 已验收”；Windows 11 x64 干净 VM/机器验收仍是后续 acceptance gate。`macos-15` 必须实际报告 `arm64`，如果未来 runner label 不再提供 Apple Silicon，架构断言应使 CI 失败，不能静默降级到 Intel。

 `MACOSX_DEPLOYMENT_TARGET=13.0` 只建立编译兼容目标，不证明应用已在 macOS 13 干净机器运行。公开发布仍需 Windows 签名，以及 macOS Developer ID 签名、公证和 staple；这些均不属于 Slice 0。

## Slice 1 首次检测平台边界

`windows-2025` 是 hosted Windows Server x64；只证明 compile/test 和 `FakePlatformAdapter` 契约，不等于 Windows 11 x64 普通用户干净机检测。

`macos-15` arm64 只证明 Apple Silicon hosted compile/test 和 fake contract；`MACOSX_DEPLOYMENT_TARGET=13.0` 不等于 macOS 13 干净机验收。

Slice 1 真实验收仍需在普通用户干净机完成：

- Windows 11 x64，Git、Claude Code、Codex CLI 分别覆盖 absent/healthy/PATH issue/broken；
- macOS 13+ Apple Silicon，覆盖同一矩阵；
- macOS 未安装 CLT 时确认检测不会弹出 Apple 安装对话框；
- 两个平台确认 `--version` timeout 后 UI 可恢复；
- UI reload、窗口重新聚焦、重复事件、sequence 缺口；
- 进程监控确认无 Node/npm、无提权、无网络、无遥测。

在上述机器验收完成前，只能报告“hosted compile/test coverage”，不能写“Windows 11/macOS 13 验收通过”。现有 `.github/workflows/ci.yml` 已运行根 `npm run check` 和 Tauri build；本切片不修改 workflow。不得增加伪造 OS 版本的 label 或成功声明。
