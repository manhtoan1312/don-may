import { useEffect, useState } from "react";
import { formatBytes } from "../../lib/format";
import { scanPath, type Scan } from "./api";
import { BigFiles } from "./BigFiles";
import { Duplicates } from "./Duplicates";

type Props = { path: string; canGoUp: boolean; onOpen: (path: string) => void; onBack: () => void };

export function ScanView({ path, canGoUp, onOpen, onBack }: Props) {
  const [scan, setScan] = useState<Scan | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    scanPath(path).then(setScan).catch((e) => setError(String(e)));
  }, [path]);

  const header = (
    <div className="toolbar">
      <button onClick={onBack}>← {canGoUp ? "Lên thư mục cha" : "Danh sách ổ đĩa"}</button>
      <strong className="path">{path}</strong>
      {scan && <span className="muted">Tổng {formatBytes(scan.total)}</span>}
    </div>
  );

  if (error) return <>{header}<p className="error">{error}</p></>;
  if (!scan) return <>{header}<p className="muted">Đang quét… ổ C có thể mất 1–2 phút.</p></>;

  return (
    <>
      {header}
      <div className="scan-grid">
        <section className="card">
          <h2>Thư mục / file nặng nhất</h2>
          <ul className="size-list">
            {scan.children.slice(0, 30).map((c) => (
              <li key={c.path}>
                <button className="link" disabled={!c.isDir} onClick={() => onOpen(c.path)} title={c.path}>
                  {c.isDir ? "📁" : "📄"} {c.name}
                </button>
                <span className="bar"><span style={{ width: `${scan.total ? (c.bytes / scan.total) * 100 : 0}%` }} /></span>
                <span className="num">{formatBytes(c.bytes)}</span>
              </li>
            ))}
          </ul>
        </section>
        <section className="card">
          <h2>Theo loại file</h2>
          <ul className="size-list">
            {scan.types.map(([kind, bytes, count]) => (
              <li key={kind}>
                <span>{kind} <small className="muted">({count} file)</small></span>
                <span className="bar"><span style={{ width: `${scan.total ? (bytes / scan.total) * 100 : 0}%` }} /></span>
                <span className="num">{formatBytes(bytes)}</span>
              </li>
            ))}
          </ul>
        </section>
      </div>
      <BigFiles files={scan.bigFiles} />
      <Duplicates path={path} />
    </>
  );
}
