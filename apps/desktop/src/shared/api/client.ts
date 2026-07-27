import { invoke } from "@tauri-apps/api/core";

import type { BootstrapState } from "./generated";

export function getBootstrapState(): Promise<BootstrapState> {
  return invoke<BootstrapState>("get_bootstrap_state");
}
