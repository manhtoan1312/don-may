import { useEffect, useState } from "react";
import { formatBytes } from "../../lib/format";
import { TrashButton } from "../../components/TrashButton";
import { deleteLeftover, findLeftovers, type App, type Leftover } from "./api";

type Props = { app: App; stillInstalled: boolean; onClose: () => void };

export function LeftoverPanel({ app, stillInstalled, onClose }: Props) {
  const [items, setItems] = useState<Leftover[] | null>(null);
  const [loading, setLoading] = useState(false);

  const search = async () => {
    setLoading(true);
    setItems(await findLeftovers(app));
    setLoading(false);
  };

  useEffect(() => {
    if (!stillInstalled) search();
    // Chỉ tự tìm một lần khi mở panel.
  }, []);

  return (
    <section className="card leftover-panel">
      <header className="card-head">
        <h2>Sau khi gỡ: {app.name}</h2>
        <button onClick={onClose} aria-label="Đóng">✕</button>
      </header>
      {stillInstalled && (
        <p className="warn-text">
          App vẫn còn trong danh sách. Nếu trình gỡ vẫn đang chạy, hãy đợi nó xong rồi bấm "Tìm đồ còn sót".
        </p>
      )}
      <button onClick={search} disabled={loading}>{loading ? "Đang tìm…" : "Tìm đồ còn sót"}</button>
      {items && items.length === 0 && <p className="success">Không thấy thư mục nào còn sót.</p>}
      {items && items.length > 0 && (
        <ul className="folder-list">
          {items.map((l) => (
            <li key={l.path} className="folder-row">
              <span title={l.path}>{l.path}</span>
              <strong className="num">{formatBytes(l.bytes)}</strong>
              <TrashButton onConfirm={() => deleteLeftover(l.path)} blocked={l.blocked} />
            </li>
          ))}
        </ul>
      )}
      <small className="muted">Chuyển vào Thùng rác nên vẫn khôi phục được nếu có gì sai.</small>
    </section>
  );
}
