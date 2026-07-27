import type {
  ObservedToolState,
  VersionStatus,
} from "../../shared/api/generated";
import type { MessageKey } from "../../shared/i18n/zh-CN";

const FACT_MESSAGE_KEYS: Record<ObservedToolState, MessageKey> = {
  absent: "status.absent",
  presentHealthy: "status.presentHealthy",
  presentPathIssue: "status.presentPathIssue",
  presentBroken: "status.presentBroken",
  unknown: "status.unknown",
};

const VERSION_MESSAGE_KEYS: Record<VersionStatus, MessageKey> = {
  current: "version.current",
  outdated: "version.outdated",
  newerThanKnown: "version.newerThanKnown",
  notComparable: "version.notComparable",
  unknown: "version.unknown",
};

export function factMessageKey(state: ObservedToolState): MessageKey {
  return FACT_MESSAGE_KEYS[state];
}

export function versionMessageKey(status: VersionStatus): MessageKey {
  return VERSION_MESSAGE_KEYS[status];
}
