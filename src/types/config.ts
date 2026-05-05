export type InstallNetworkMode = "none" | "system_proxy" | "manual_proxy";

export type NpmRegistryOption = "default" | "npmmirror" | "custom";

export type InstallNetworkConfig = {
  mode: InstallNetworkMode;
  proxyUrl?: string;
  npmRegistry: NpmRegistryOption;
  customNpmRegistry?: string;
};

export type CcSwitchDownloadSourceKind = "direct_exe";

export type CcSwitchDownloadSource = {
  name: string;
  url: string;
  priority: number;
  enabled: boolean;
  kind: CcSwitchDownloadSourceKind;
  sha256?: string;
  minFileSizeBytes?: number;
};

export type AppConfig = {
  installNetwork: InstallNetworkConfig;
  ccswitchPath?: string;
  ccswitchDownloadSources: CcSwitchDownloadSource[];
  subscriptionPageUrl?: string;
};

export type AppConfigPatch = {
  installNetwork?: Partial<InstallNetworkConfig>;
  ccswitchPath?: string;
  ccswitchDownloadSources?: CcSwitchDownloadSource[];
  subscriptionPageUrl?: string;
};

export const defaultAppConfig: AppConfig = {
  installNetwork: {
    mode: "none",
    npmRegistry: "default",
  },
  ccswitchDownloadSources: [],
  subscriptionPageUrl: undefined,
};
