import { invoke } from "@tauri-apps/api/core";

export type Drive = { mount: string; name: string; total: number; available: number };
export type Verdict = "khong_duoc_xoa" | "co_the_xoa" | "can_nhac" | "khong_ro";
export type Child = { path: string; name: string; isDir: boolean; bytes: number };
export type BigFile = { path: string; bytes: number; ageDays: number; verdict: Verdict; reason: string };
/** [loại file, tổng byte, số file] */
export type TypeStat = [string, number, number];
export type Scan = { path: string; total: number; children: Child[]; bigFiles: BigFile[]; types: TypeStat[] };
export type DupGroup = { bytesEach: number; files: { path: string; ageDays: number }[] };
export type FileVerdict = { what: string; safeToDelete: "co" | "khong" | "khong_ro"; reason: string };

export const VERDICT_LABEL: Record<Verdict, string> = {
  khong_duoc_xoa: "Không được xóa",
  co_the_xoa: "Có thể xóa",
  can_nhac: "Cân nhắc",
  khong_ro: "Chưa rõ",
};

export const listDrives = () => invoke<Drive[]>("list_drives");
export const scanPath = (path: string) => invoke<Scan>("scan_path", { path });
export const deleteFile = (path: string) => invoke<void>("delete_file", { path });
export const findDuplicates = (path: string) => invoke<DupGroup[]>("find_duplicates", { path });
export const explainFile = (path: string) => invoke<FileVerdict>("explain_file", { path });
