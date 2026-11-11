import { invoke } from "@tauri-apps/api/core";

export type ProcInfo = {
  pid: number;
  parentPid: number | null;
  name: string;
  exe: string | null;
  cmd: string;
  cpu: number;
  memory: number;
  flags: string[];
};

export type Snapshot = {
  cpu: number;
  memUsed: number;
  memTotal: number;
  processes: ProcInfo[];
};

export const getSnapshot = () => invoke<Snapshot>("snapshot");
export const killProcess = (pid: number) => invoke<void>("kill_process", { pid });
export type Risk = "thap" | "trung_binh" | "cao";

export type Verdict = {
  info: string;
  program: string;
  risk: Risk;
  aiRisk: Risk;
  reasons: string[];
  advice: string;
};

export const analyzeProcess = (pid: number) => invoke<Verdict>("analyze_process", { pid });
