import { useEffect, useState } from "react";
import { formatBytes } from "../../lib/format";
import { listDrives, type Drive } from "./api";
import { ScanView } from "./ScanView";

function DriveList({ onScan }: { onScan: (path: string) => void }) {
  const [drives, setDrives] = useState<Drive[] | null>(null);
  useEffect(() => {
    listDrives().then(setDrives);
  }, []);
  if (!drives) return <p className="muted">Đang đọc ổ đĩa…</p>;
  return (
    <div className="drives">
      {drives.map((d) => {
        const used = d.total - d.available;
        const pct = d.total ? (used / d.total) * 100 : 0;
        return (
          <section key={d.mount} className="card">
            <header className="card-head">
              <h2>{d.mount} {d.name && <small className="muted">{d.name}</small>}</h2>
              <button className="primary" onClick={() => onScan(d.mount)}>Quét ổ này</button>
            </header>
            <div className="meter-track"><div className={`meter-fill ${pct >= 90 ? "danger" : pct >= 75 ? "warn" : "ok"}`} style={{ width: `${pct}%` }} /></div>
            <small className="muted">Đã dùng {formatBytes(used)} / {formatBytes(d.total)} · còn trống {formatBytes(d.available)}</small>
          </section>
        );
      })}
    </div>
  );
}

export function FilesView() {
  // Ngăn xếp đường dẫn để quay lại thư mục cha.
  const [stack, setStack] = useState<string[]>([]);
  const path = stack[stack.length - 1];
  return (
    <div className="files">
      {path == null ? (
        <DriveList onScan={(p) => setStack([p])} />
      ) : (
        <ScanView
          key={path}
          path={path}
          canGoUp={stack.length > 1}
          onOpen={(p) => setStack([...stack, p])}
          onBack={() => setStack(stack.slice(0, -1))}
        />
      )}
    </div>
  );
}
