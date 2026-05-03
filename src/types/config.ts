export type InstallNetworkMode = "none" | "system_proxy" | "manual_proxy";

export type NpmRegistryOption = "default" | "npmmirror" | "custom";

export type InstallNetworkConfig = {
  mode: InstallNetworkMode;
  proxyUrl?: string;
  npmRegistry: NpmRegistryOption;
  customNpmRegistry?: string;
};

export type AppConfig = {
  installNetwork: InstallNetworkConfig;
  ccswitchPath?: string;
};

export type AppConfigPatch = {
  installNetwork?: Partial<InstallNetworkConfig>;
  ccswitchPath?: string;
};

export const defaultAppConfig: AppConfig = {
  installNetwork: {
    mode: "none",
    npmRegistry: "default",
  },
};
