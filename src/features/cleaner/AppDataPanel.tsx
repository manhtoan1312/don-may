import { useState } from "react";
import { formatBytes } from "../../lib/format";
import { TrashButton } from "../../components/TrashButton";
import { deleteLeftover } from "../apps/api";
import { appdataFolders, explainFolder, type Folder, type FolderVerdict, type Safe } from "./api";

const SAFE_LABEL: Record<Safe, string> = { co: "Có thể xóa", khong: "Không nên xóa", khong_ro: "Không rõ" };

function FolderRow({ folder }: { folder: Folder }) {
  const [verdict, setVerdict] = useState<FolderVerdict | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const ask = async () => {
    setLoading(true);
    setError(null);
    try {
      setVerdict(await explainFolder(folder.path));
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  return (
    <li>
      <div className="folder-row">
        <span title={folder.path}>
          {folder.name} <small className="muted">({folder.root})</small>
        </span>
        <strong className="num">{formatBytes(folder.bytes)}</strong>
        <button onClick={ask} disabled={loading}>{loading ? "Đang hỏi…" : "Hỏi AI"}</button>
      </div>
      {!folder.blocked && (
        <div className="orphan">
          <span className="safe safe-khong_ro">Có thể là đồ còn sót</span> Không khớp app nào đang cài.{" "}
          <TrashButton onConfirm={() => deleteLeftover(folder.path)} />
        </div>
      )}
      {error && <small className="error">{error}</small>}
      {verdict && (
        <div className="folder-verdict">
          <span className={`safe safe-${verdict.safeToDelete}`}>{SAFE_LABEL[verdict.safeToDelete]}</span>{" "}
          <strong>{verdict.app}</strong>: {verdict.reason}
        </div>
      )}
    </li>
  );
}

export function AppDataPanel() {
  const [folders, setFolders] = useState<Folder[] | null>(null);
  const [loading, setLoading] = useState(false);

  const load = async () => {
    setLoading(true);
    setFolders(await appdataFolders());
    setLoading(false);
  };

  return (
    <section className="card">
      <header className="card-head">
        <h2>Thư mục lớn trong AppData</h2>
        <button onClick={load} disabled={loading}>{loading ? "Đang tính dung lượng…" : "Thống kê"}</button>
      </header>
      <small className="muted">
        Chỉ thư mục không khớp app nào đang cài, không có chương trình nào chạy từ đó và không được sửa
        trong 30 ngày mới có nút xóa (vào Thùng rác). Ý kiến AI chỉ để tham khảo.
      </small>
      {folders && (
        <ul className="folder-list">
          {folders.map((f) => <FolderRow key={f.path} folder={f} />)}
        </ul>
      )}
    </section>
  );
}
