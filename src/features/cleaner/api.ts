import { invoke } from "@tauri-apps/api/core";

export type Stats = { bytes: number; files: number; failed: number };
export type JunkItem = Stats & { id: string; name: string; desc: string };

export type Folder = {
  path: string;
  name: string;
  root: "Local" | "Roaming";
  bytes: number;
  /** null = đồ còn sót, được phép chuyển vào Thùng rác. */
  blocked: string | null;
};
export type Safe = "co" | "khong" | "khong_ro";
export type FolderVerdict = {
  app: string;
  kind: "cache" | "du_lieu" | "cau_hinh" | "khong_ro";
  safeToDelete: Safe;
  reason: string;
};

export const scanJunk = () => invoke<JunkItem[]>("scan_junk");
export const cleanJunk = (ids: string[]) => invoke<Stats>("clean_junk", { ids });
export const appdataFolders = () => invoke<Folder[]>("appdata_folders");
export const explainFolder = (path: string) => invoke<FolderVerdict>("explain_folder", { path });
