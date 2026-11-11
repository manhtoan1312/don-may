const UNITS = ["B", "KB", "MB", "GB", "TB"];

export function formatBytes(bytes: number) {
  let i = 0;
  while (bytes >= 1024 && i < UNITS.length - 1) {
    bytes /= 1024;
    i++;
  }
  return `${bytes.toFixed(i ? 1 : 0)} ${UNITS[i]}`;
}

export const formatPercent = (n: number) => `${n.toFixed(1)}%`;

/** "20260115" (cách Windows lưu) -> "15/01/2026". */
export function formatInstallDate(d: string | null) {
  const m = d?.match(/^(\d{4})(\d{2})(\d{2})$/);
  return m ? `${m[3]}/${m[2]}/${m[1]}` : null;
}
