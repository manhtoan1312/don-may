import { useState } from "react";
import { TrashButton } from "../../components/TrashButton";
import { formatBytes } from "../../lib/format";
import { deleteFile, findDuplicates, type DupGroup } from "./api";

function Group({ group }: { group: DupGroup }) {
  const [deleted, setDeleted] = useState<Set<string>>(new Set());
  const remaining = group.files.length - deleted.size;
  return (
    <li className="app-row">
      <strong>{group.files.length} bản giống hệt nhau · {formatBytes(group.bytesEach)} mỗi bản</strong>
      <ul className="dup-files">
        {group.files.map((f) => (
          <li key={f.path} className="file-main">
            <span className="path" title={f.path}>{f.path}</span>
            <small className="muted num">{f.ageDays} ngày</small>
            <TrashButton
              onConfirm={() => deleteFile(f.path)}
              onDone={() => setDeleted(new Set(deleted).add(f.path))}
              // Luôn giữ lại ít nhất 1 bản.
              blocked={!deleted.has(f.path) && remaining <= 1 ? "bản cuối cùng" : null}
            />
          </li>
        ))}
      </ul>
    </li>
  );
}

export function Duplicates({ path }: { path: string }) {
  const [groups, setGroups] = useState<DupGroup[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = async () => {
    setLoading(true);
    setError(null);
    try {
      setGroups(await findDuplicates(path));
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  const wasted = (groups ?? []).reduce((s, g) => s + g.bytesEach * (g.files.length - 1), 0);

  return (
    <section className="card">
      <header className="card-head">
        <h2>File trùng nhau (≥ 1 MB)</h2>
        <button onClick={run} disabled={loading}>{loading ? "Đang so sánh nội dung…" : "Tìm file trùng"}</button>
      </header>
      <small className="muted">So sánh từng byte nội dung, không chỉ tên. Luôn giữ lại ít nhất 1 bản.</small>
      {error && <p className="error">{error}</p>}
      {groups && groups.length === 0 && <p className="success">Không có file trùng.</p>}
      {groups && groups.length > 0 && (
        <>
          <p>Có thể giải phóng khoảng <strong>{formatBytes(wasted)}</strong>.</p>
          <ul className="app-list">{groups.map((g) => <Group key={g.files[0].path} group={g} />)}</ul>
        </>
      )}
    </section>
  );
}
