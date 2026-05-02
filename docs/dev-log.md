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
