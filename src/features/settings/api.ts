import { invoke } from "@tauri-apps/api/core";

export type Settings = {
  alertsEnabled: boolean;
  cpuThreshold: number;
  ramThreshold: number;
  sustainSecs: number;
  cooldownMins: number;
  ollamaModel: string;
};

export const getSettings = () => invoke<Settings>("get_settings");
export const saveSettings = (settings: Settings) => invoke<Settings>("save_settings", { settings });
export const testNotification = () => invoke<void>("test_notification");
