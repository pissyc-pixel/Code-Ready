import { useEffect, useMemo, useState } from "react";
import "./App.css";
import AppShell from "./components/shell/AppShell";
import SideNav, { type SideNavItem } from "./components/shell/SideNav";
import TopBar from "./components/shell/TopBar";
import { mockAiTools, mockBaseTools } from "./data/mockTools";
import { useDetectEvents } from "./hooks/useDetectEvents";
import { useInstallEvents } from "./hooks/useInstallEvents";
import {
  cancelInstall,
  detectAllTools,
  detectTool,
  exportDiagnosticsLogZip,
  getConfig,
  getLogPreview,
  installLatestTool,
  installTool,
  isAdmin,
  openCcSwitch,
  openFullLogFile,
  openLogDirectory,
  openSubscriptionPage,
  reinstallTool,
  restartAsAdmin,
  updateConfig,
} from "./lib/api";
import AiToolsPage from "./pages/AiToolsPage";
import CCSwitchPage from "./pages/CCSwitchPage";
import DashboardPage from "./pages/DashboardPage";
import EnvPage from "./pages/EnvPage";
import LogsPage from "./pages/LogsPage";
import SettingsPage from "./pages/SettingsPage";
import {
  defaultAppConfig,
  type AppConfig,
  type InstallNetworkConfig,
} from "./types/config";
import type { InstallStatusEvent, InstallTaskViewState } from "./types/install";
import type { ToolId, ToolStatus } from "./types/tool";

const initialRows = [...mockBaseTools, ...mockAiTools];
const LOG_VIEWER_LINE_LIMIT = 5000;

type AppViewId =
  | "dashboard"
  | "environment"
  | "aiTools"
  | "ccswitch"
  | "logs"
  | "settings";

const APP_NAV_ITEMS: SideNavItem[] = [
  { id: "dashboard", label: "总览", caption: "环境概览" },
  { id: "environment", label: "基础环境", caption: "Git · Node · Python" },
  { id: "aiTools", label: "AI 工具", caption: "Claude · Codex · OpenCode" },
  { id: "ccswitch", label: "ccSwitch", caption: "路径与启动" },
  { id: "logs", label: "日志", caption: "诊断与导出" },
  { id: "settings", label: "设置", caption: "网络与权限" },
];

const VIEW_META: Record<AppViewId, { title: string; description: string }> = {
  dashboard: {
    title: "总览",
    description: "保留真实逻辑，只迁移壳层和首页信息分发。",
  },
  environment: {
    title: "基础环境",
    description: "保留原有检测、安装、PATH 修复与重新检测动作。",
  },
  aiTools: {
    title: "AI 工具",
    description: "保留 Claude / Codex / OpenCode / ccSwitch 的真实安装与检测逻辑。",
  },
  ccswitch: {
    title: "ccSwitch",
    description: "路径保存、启动入口和订阅页 URL 继续走现有状态与 invoke 调用。",
  },
  logs: {
    title: "日志与诊断",
    description: "实时日志、完整日志目录和诊断压缩包继续复用原有数据链路。",
  },
  settings: {
    title: "设置",
    description: "安装网络与权限信息继续保留在现有配置逻辑中。",
  },
};

function App() {
  const [rows, setRows] = useState<ToolStatus[]>(() =>
    initialRows.map((tool) => ({ ...tool, status: "checking" })),
  );
  const [activeView, setActiveView] = useState<AppViewId>("dashboard");
  const [isDetectingAll, setIsDetectingAll] = useState(false);
  const [installState, setInstallState] = useState<InstallTaskViewState>({
    isInstalling: false,
  });
  const [config, setConfig] = useState<AppConfig>(defaultAppConfig);
  const [ccswitchPathInput, setCcswitchPathInput] = useState("");
  const [subscriptionPageUrlInput, setSubscriptionPageUrlInput] = useState("");
  const [quickActionMessage, setQuickActionMessage] = useState("");
  const [logLines, setLogLines] = useState<string[]>([]);
  const [logActionMessage, setLogActionMessage] = useState("");
  const [isExportingDiagnostics, setIsExportingDiagnostics] = useState(false);
  const [pathRepairTool, setPathRepairTool] = useState<ToolStatus | null>(null);
  const [pathRepairCopyState, setPathRepairCopyState] = useState<
    "idle" | "copied" | "failed"
  >("idle");
  const [adminState, setAdminState] = useState({
    checked: false,
    isAdmin: false,
  });

  useDetectEvents((event) => {
    applyResult(event.result);
  });

  useInstallEvents({
    onProgress: applyLogProgress,
    onStatus: applyInstallStatus,
  });

  useEffect(() => {
    void runDetectAll();
    void loadConfig();
    void loadPrivilege();
    void loadLogPreview();
  }, []);

  const groupedRows = useMemo(
    () => ({
      base: rows.filter((row) => row.category === "base"),
      ai: rows.filter((row) => row.category === "ai"),
    }),
    [rows],
  );
  const ccswitchRow = rows.find((row) => row.id === "ccswitch");
  const activeInstallRow = rows.find((row) => row.id === installState.activeToolId);
  const latestLogLine = logLines.length > 0 ? logLines[logLines.length - 1] : undefined;
  const currentViewMeta = VIEW_META[activeView];
  const npmRegistryLabel =
    config.installNetwork.npmRegistry === "custom"
      ? config.installNetwork.customNpmRegistry ?? "custom"
      : config.installNetwork.npmRegistry;

  function applyResult(result: ToolStatus) {
    setRows((currentRows) =>
      currentRows.map((row) => {
        if (row.id !== result.id) {
          return row;
        }

        if (
          row.lastCheckedAt &&
          result.lastCheckedAt &&
          row.lastCheckedAt > result.lastCheckedAt
        ) {
          return row;
        }

        return result;
      }),
    );
  }

  function applyInstallStatus(event: InstallStatusEvent) {
    setRows((currentRows) =>
      currentRows.map((row) =>
        row.id === event.toolId
          ? {
              ...row,
              status: event.status,
              errorMessage: event.errorMessage,
              suggestion: event.suggestion ?? row.suggestion,
              lastCheckedAt: event.timestamp,
            }
          : row,
      ),
    );

    setInstallState({
      activeToolId:
        event.phase === "started" || event.phase === "running"
          ? event.toolId
          : undefined,
      isInstalling: event.phase === "started" || event.phase === "running",
    });
  }

  function applyLogProgress(event: {
    stream: string;
    line: string;
    timestamp: string;
  }) {
    appendLogLine(`[${event.timestamp}] [${event.stream}] ${event.line}`);
  }

  function appendLogLine(line: string) {
    setLogLines((currentLines) =>
      [...currentLines, line].slice(-LOG_VIEWER_LINE_LIMIT),
    );
  }

  async function loadConfig() {
    const nextConfig = await getConfig();
    setConfig(nextConfig);
    setCcswitchPathInput(nextConfig.ccswitchPath ?? "");
    setSubscriptionPageUrlInput(nextConfig.subscriptionPageUrl ?? "");
  }

  async function loadPrivilege() {
    const nextIsAdmin = await isAdmin();
    setAdminState({
      checked: true,
      isAdmin: nextIsAdmin,
    });
  }

  async function loadLogPreview() {
    try {
      const preview = await getLogPreview(LOG_VIEWER_LINE_LIMIT);
      setLogLines(preview.slice(-LOG_VIEWER_LINE_LIMIT));
      setLogActionMessage("");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`日志预览暂时不可用：${message}`);
    }
  }

  async function runRefreshLogPreview() {
    await loadLogPreview();
  }

  async function runDetectAll() {
    setIsDetectingAll(true);
    setRows((currentRows) =>
      currentRows.map((row) => ({
        ...row,
        status: "checking",
        errorMessage: undefined,
      })),
    );

    try {
      const results = await detectAllTools();
      results.forEach((result) => {
        applyResult(result);
      });
    } finally {
      setIsDetectingAll(false);
    }
  }

  async function runDetectOne(toolId: ToolId) {
    setRows((currentRows) =>
      currentRows.map((row) =>
        row.id === toolId
          ? { ...row, status: "checking", errorMessage: undefined }
          : row,
      ),
    );

    const result = await detectTool(toolId);
    applyResult(result);
  }

  async function runInstall(toolId: ToolId) {
    try {
      await installTool(toolId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === toolId ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function runReinstall(toolId: ToolId) {
    try {
      await reinstallTool(toolId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === toolId ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function runInstallLatest(toolId: ToolId) {
    try {
      await installLatestTool(toolId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === toolId ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function runCancelInstall() {
    try {
      await cancelInstall();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === installState.activeToolId
            ? { ...row, errorMessage: message }
            : row,
        ),
      );
    }
  }

  async function runRestartAsAdmin() {
    await restartAsAdmin();
  }

  async function saveCcSwitchPath() {
    const nextConfig = await updateConfig({
      ccswitchPath: ccswitchPathInput,
    });
    setConfig(nextConfig);
    setCcswitchPathInput(nextConfig.ccswitchPath ?? "");
    const result = await detectTool("ccswitch");
    applyResult(result);
  }

  async function saveSubscriptionPageUrl() {
    const nextConfig = await updateConfig({
      subscriptionPageUrl: subscriptionPageUrlInput,
    });
    setConfig(nextConfig);
    setSubscriptionPageUrlInput(nextConfig.subscriptionPageUrl ?? "");
    setQuickActionMessage("节点订阅网页已保存。");
  }

  async function saveInstallNetwork(nextInstallNetwork: InstallNetworkConfig) {
    const nextConfig = await updateConfig({
      installNetwork: nextInstallNetwork,
    });
    setConfig(nextConfig);
  }

  async function runOpenCcSwitch() {
    try {
      await openCcSwitch();
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === "ccswitch" ? { ...row, errorMessage: undefined } : row,
        ),
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setRows((currentRows) =>
        currentRows.map((row) =>
          row.id === "ccswitch" ? { ...row, errorMessage: message } : row,
        ),
      );
    }
  }

  async function runOpenSubscriptionPage() {
    try {
      await openSubscriptionPage();
      setQuickActionMessage("已用默认浏览器打开节点订阅网页。");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setQuickActionMessage(message);
    }
  }

  async function runOpenFullLogFile() {
    try {
      await openFullLogFile();
      setLogActionMessage("已打开完整日志文件。");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`打开完整日志文件失败：${message}`);
    }
  }

  async function runOpenLogDirectory() {
    try {
      await openLogDirectory();
      setLogActionMessage("已打开日志目录。");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`打开日志目录失败：${message}`);
    }
  }

  async function runExportDiagnosticsLogZip() {
    setIsExportingDiagnostics(true);
    setLogActionMessage("正在导出诊断 zip...");
    try {
      const result = await exportDiagnosticsLogZip();
      setLogActionMessage(`诊断 zip 已导出：${result.path}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLogActionMessage(`导出诊断 zip 失败：${message}`);
    } finally {
      setIsExportingDiagnostics(false);
    }
  }

  async function copyPathRepairCommand(command: string) {
    try {
      await navigator.clipboard.writeText(command);
      setPathRepairCopyState("copied");
    } catch {
      setPathRepairCopyState("failed");
    }
  }

  function openPathRepair(tool: ToolStatus) {
    setPathRepairTool(tool);
    setPathRepairCopyState("idle");
  }

  function closePathRepair() {
    setPathRepairTool(null);
    setPathRepairCopyState("idle");
  }

  function renderEnvironmentPage() {
    return (
      <EnvPage
        rows={groupedRows.base}
        isDetectingAll={isDetectingAll}
        installState={installState}
        activeInstallTool={activeInstallRow}
        latestLogLine={latestLogLine}
        onDetectAll={runDetectAll}
        onDetect={runDetectOne}
        onInstall={runInstall}
        onReinstall={runReinstall}
        onInstallLatest={runInstallLatest}
        onCancelInstall={runCancelInstall}
        onOpenPathRepair={openPathRepair}
        onNavigateLogs={() => setActiveView("logs")}
        onOpenLogDirectory={runOpenLogDirectory}
      />
    );
  }

  function renderAiToolsPage() {
    return (
      <AiToolsPage
        rows={groupedRows.ai}
        config={config}
        isDetectingAll={isDetectingAll}
        installState={installState}
        activeInstallTool={activeInstallRow}
        latestLogLine={latestLogLine}
        canOpenCcSwitch={Boolean(ccswitchRow?.executablePath)}
        onDetectAll={runDetectAll}
        onDetect={runDetectOne}
        onInstall={runInstall}
        onReinstall={runReinstall}
        onInstallLatest={runInstallLatest}
        onCancelInstall={runCancelInstall}
        onOpenPathRepair={openPathRepair}
        onOpenLogDirectory={runOpenLogDirectory}
        onNavigateLogs={() => setActiveView("logs")}
        onOpenCcSwitch={runOpenCcSwitch}
        onOpenSubscriptionPage={runOpenSubscriptionPage}
      />
    );
  }

  function renderCcSwitchPage() {
    return (
      <CCSwitchPage
        row={ccswitchRow}
        ccswitchPathInput={ccswitchPathInput}
        subscriptionPageUrlInput={subscriptionPageUrlInput}
        downloadSourceCount={config.ccswitchDownloadSources.length}
        quickActionMessage={quickActionMessage}
        onPathInputChange={setCcswitchPathInput}
        onSubscriptionUrlInputChange={setSubscriptionPageUrlInput}
        onSavePath={saveCcSwitchPath}
        onSaveSubscriptionPageUrl={saveSubscriptionPageUrl}
        onOpenCcSwitch={runOpenCcSwitch}
        onOpenSubscriptionPage={runOpenSubscriptionPage}
        onDetectCcSwitch={() => runDetectOne("ccswitch")}
      />
    );
  }

  function renderLogsPage() {
    return (
      <LogsPage
        logLines={logLines}
        logActionMessage={logActionMessage}
        isExportingDiagnostics={isExportingDiagnostics}
        logViewerLineLimit={LOG_VIEWER_LINE_LIMIT}
        onRefreshPreview={runRefreshLogPreview}
        onOpenFullLogFile={runOpenFullLogFile}
        onOpenLogDirectory={runOpenLogDirectory}
        onExportDiagnostics={runExportDiagnosticsLogZip}
      />
    );
  }

  function renderSettingsPage() {
    return (
      <SettingsPage
        config={config}
        adminChecked={adminState.checked}
        isAdmin={adminState.isAdmin}
        onSaveInstallNetwork={saveInstallNetwork}
        onRestartAsAdmin={runRestartAsAdmin}
      />
    );
  }

  function renderCurrentPage() {
    switch (activeView) {
      case "dashboard":
        return (
          <DashboardPage
            rows={rows}
            groupedRows={groupedRows}
            config={config}
            installState={installState}
            activeInstallTool={activeInstallRow}
            latestLogLine={latestLogLine}
            logLines={logLines}
            adminChecked={adminState.checked}
            isAdmin={adminState.isAdmin}
            quickActionMessage={quickActionMessage}
            logActionMessage={logActionMessage}
            canOpenCcSwitch={Boolean(ccswitchRow?.executablePath)}
            onDetectAll={runDetectAll}
            onDetect={runDetectOne}
            onInstall={runInstall}
            onReinstall={runReinstall}
            onCancelInstall={runCancelInstall}
            onRestartAsAdmin={runRestartAsAdmin}
            onOpenCcSwitch={runOpenCcSwitch}
            onOpenSubscriptionPage={runOpenSubscriptionPage}
            onExportDiagnostics={runExportDiagnosticsLogZip}
            onNavigate={(pageId) => setActiveView(pageId as AppViewId)}
          />
        );
      case "environment":
        return renderEnvironmentPage();
      case "aiTools":
        return renderAiToolsPage();
      case "ccswitch":
        return renderCcSwitchPage();
      case "logs":
        return renderLogsPage();
      case "settings":
        return renderSettingsPage();
      default:
        return null;
    }
  }

  return (
    <>
      <AppShell
        sidebar={
          <SideNav
            items={APP_NAV_ITEMS}
            activeItemId={activeView}
            onSelect={(id) => setActiveView(id as AppViewId)}
          />
        }
        topbar={
          <TopBar
            title={currentViewMeta.title}
            description={currentViewMeta.description}
            npmRegistryLabel={npmRegistryLabel}
            isAdmin={adminState.isAdmin}
            activeInstallLabel={activeInstallRow?.name}
          />
        }
      >
        {renderCurrentPage()}
      </AppShell>

      {pathRepairTool?.executablePath ? (
        <PathRepairModal
          tool={pathRepairTool}
          directory={pathDirectory(pathRepairTool.executablePath)}
          command={pathRepairCommand(pathDirectory(pathRepairTool.executablePath))}
          copyState={pathRepairCopyState}
          onCopy={copyPathRepairCommand}
          onClose={closePathRepair}
        />
      ) : null}
    </>
  );
}

type PathRepairModalProps = {
  tool: ToolStatus;
  directory: string;
  command: string;
  copyState: "idle" | "copied" | "failed";
  onCopy: (command: string) => Promise<void>;
  onClose: () => void;
};

function PathRepairModal({
  tool,
  directory,
  command,
  copyState,
  onCopy,
  onClose,
}: PathRepairModalProps) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section
        className="path-repair-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="path-repair-title"
      >
        <div className="modal-header">
          <div>
            <p className="eyebrow">PATH 缺失</p>
            <h2 id="path-repair-title">可执行文件存在，但终端 PATH 没刷新</h2>
          </div>
          <button type="button" className="ghost-button" onClick={onClose}>
            关闭
          </button>
        </div>

        <div className="repair-grid">
          <div className="network-stat">
            <span>工具</span>
            <strong>{tool.name}</strong>
          </div>
          <div className="network-stat">
            <span>可执行文件</span>
            <strong className="mono breakable">{tool.executablePath}</strong>
          </div>
          <div className="network-stat">
            <span>需要加入 PATH 的目录</span>
            <strong className="mono breakable">{directory}</strong>
          </div>
        </div>

        <p className="network-warning">
          本客户端不会自动修改 PATH。你可以先重启终端；如果仍然无效，再复制下面的
          PowerShell 命令手动写入用户 PATH。
        </p>

        <ol className="manual-steps">
          <li>选项 A：关闭并重新打开 PowerShell / Windows Terminal。</li>
          <li>选项 B：复制下面的用户级 PATH 命令，手动在 PowerShell 中执行。</li>
          <li>执行后请重启所有终端，让 PATH 刷新。</li>
          <li>回到本应用，再点击“重新检测”。</li>
        </ol>

        <pre className="command-preview">
          <code>{command}</code>
        </pre>

        <div className="action-group">
          <button type="button" onClick={() => void onCopy(command)}>
            复制
          </button>
          {copyState === "copied" ? (
            <span className="copy-state">已复制到剪贴板。</span>
          ) : null}
          {copyState === "failed" ? (
            <span className="copy-state error">复制失败，请手动选择命令内容。</span>
          ) : null}
        </div>
      </section>
    </div>
  );
}

function pathDirectory(executablePath: string): string {
  const slashIndex = Math.max(
    executablePath.lastIndexOf("\\"),
    executablePath.lastIndexOf("/"),
  );
  return slashIndex >= 0 ? executablePath.slice(0, slashIndex) : executablePath;
}

function pathRepairCommand(directory: string): string {
  const escapedDirectory = directory.replace(/'/g, "''");
  return [
    `$dir = '${escapedDirectory}'`,
    `$userPath = [Environment]::GetEnvironmentVariable("Path", "User")`,
    `$parts = @($userPath -split ';' | Where-Object { $_ })`,
    `if ($parts -notcontains $dir) {`,
    `  $next = (@($parts) + $dir) -join ';'`,
    `  [Environment]::SetEnvironmentVariable("Path", $next, "User")`,
    `}`,
  ].join("\n");
}

export default App;
