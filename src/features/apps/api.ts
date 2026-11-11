import { invoke } from "@tauri-apps/api/core";

export type App = {
  id: string;
  name: string;
  version: string | null;
  publisher: string | null;
  installDate: string | null;
  sizeKb: number;
  installLocation: string | null;
};

export type Leftover = { path: string; bytes: number; blocked: string | null };

export type Advice = "giu" | "co_the_go" | "nen_go" | "khong_ro";
export type AppVerdict = { purpose: string; advice: Advice; reason: string; overridden: boolean };

export const listApps = () => invoke<App[]>("list_apps");
/** Trả về true nếu app vẫn còn sau khi trình gỡ đóng. */
export const uninstallApp = (id: string) => invoke<boolean>("uninstall_app", { id });
export const findLeftovers = (app: App) =>
  invoke<Leftover[]>("find_leftovers", {
    name: app.name,
    publisher: app.publisher,
    installLocation: app.installLocation,
  });
export const deleteLeftover = (path: string) => invoke<void>("delete_leftover", { path });
export const explainApp = (id: string) => invoke<AppVerdict>("explain_app", { id });
