import { useState } from "react";
import { formatBytes } from "../../lib/format";
import { cleanJunk, scanJunk, type JunkItem, type Stats } from "./api";

export function JunkPanel() {
  const [items, setItems] = useState<JunkItem[] | null>(null);
  const [checked, setChecked] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState<"scan" | "clean" | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [result, setResult] = useState<Stats | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = async (kind: "scan" | "clean", job: () => Promise<void>) => {
    setBusy(kind);
    setError(null);
    try {
      await job();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(null);
    }
  };

  const scan = () =>
    run("scan", async () => {
      const list = await scanJunk();
      setItems(list);
      setChecked(new Set(list.filter((i) => i.bytes > 0).map((i) => i.id)));
    });

  const clean = async () => {
    if (!confirming) return setConfirming(true);
    setConfirming(false);
    await run("clean", async () => setResult(await cleanJunk([...checked])));
    await scan();
  };

  const toggle = (id: string) => {
    const next = new Set(checked);
    if (!next.delete(id)) next.add(id);
    setChecked(next);
  };

  const total = (items ?? []).filter((i) => checked.has(i.id)).reduce((s, i) => s + i.bytes, 0);

  return (
    <section className="card">
      <header className="card-head">
        <h2>File rác</h2>
        <button onClick={() => { setResult(null); scan(); }} disabled={busy != null}>{busy === "scan" ? "Đang quét…" : "Quét"}</button>
      </header>
      {error && <p className="error">{error}</p>}
      {result && (
        <p className="success">
          Đã dọn {formatBytes(result.bytes)} ({result.files} file).
          {result.failed > 0 && ` ${result.failed} file đang được dùng hoặc thiếu quyền nên được giữ lại.`}
        </p>
      )}
      {!items && <p className="muted">Bấm "Quét" để xem máy có bao nhiêu rác.</p>}
      {items && (
        <>
          <ul className="junk-list">
            {items.map((i) => (
              <li key={i.id}>
                <label>
                  <input type="checkbox" checked={checked.has(i.id)} onChange={() => toggle(i.id)} disabled={!i.bytes} />
                  <span className="junk-name">{i.name}</span>
                  <strong className="num">{formatBytes(i.bytes)}</strong>
                </label>
                <small className="muted">{i.desc}</small>
              </li>
            ))}
          </ul>
          <div className="settings-actions">
            <button className={confirming ? "danger" : "primary"} onClick={clean} onBlur={() => setConfirming(false)}
              disabled={busy != null || total === 0}>
              {busy === "clean" ? "Đang dọn…" : confirming ? `Xóa vĩnh viễn ${formatBytes(total)}?` : `Dọn ${formatBytes(total)}`}
            </button>
            <small className="muted">Cache và file tạm sẽ bị xóa hẳn (app tự tạo lại khi cần), không qua Thùng rác.</small>
          </div>
        </>
      )}
    </section>
  );
}
