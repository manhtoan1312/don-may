import { useEffect, useMemo, useState } from "react";
import { listApps, uninstallApp, type App } from "./api";
import { AppRow } from "./AppRow";
import { LeftoverPanel } from "./LeftoverPanel";

type SortKey = "name" | "size" | "date";
const SORTERS: Record<SortKey, (a: App, b: App) => number> = {
  name: (a, b) => a.name.localeCompare(b.name),
  size: (a, b) => b.sizeKb - a.sizeKb,
  date: (a, b) => (b.installDate ?? "").localeCompare(a.installDate ?? ""),
};

export function AppsView() {
  const [apps, setApps] = useState<App[] | null>(null);
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SortKey>("name");
  const [uninstalling, setUninstalling] = useState<string | null>(null);
  const [after, setAfter] = useState<{ app: App; stillInstalled: boolean } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = () => listApps().then(setApps).catch((e) => setError(String(e)));
  useEffect(() => {
    refresh();
  }, []);

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase();
    return (apps ?? [])
      .filter((a) => !q || `${a.name} ${a.publisher ?? ""}`.toLowerCase().includes(q))
      .sort(SORTERS[sort]);
  }, [apps, query, sort]);

  const uninstall = async (app: App) => {
    setUninstalling(app.id);
    setError(null);
    try {
      const stillInstalled = await uninstallApp(app.id);
      setAfter({ app, stillInstalled });
      await refresh();
    } catch (e) {
      setError(String(e));
    }
    setUninstalling(null);
  };

  if (!apps) return <p className="muted">{error ?? "Đang đọc danh sách app…"}</p>;

  return (
    <div className="apps">
      <div className="toolbar">
        <input placeholder="Tìm app hoặc nhà phát hành…" value={query} onChange={(e) => setQuery(e.target.value)} />
        <label>
          Sắp xếp
          <select value={sort} onChange={(e) => setSort(e.target.value as SortKey)}>
            <option value="name">Tên</option>
            <option value="size">Dung lượng</option>
            <option value="date">Ngày cài</option>
          </select>
        </label>
        <span className="muted">{rows.length} app</span>
      </div>
      {error && <p className="error">{error}</p>}
      {uninstalling && <p className="muted">Đang chạy trình gỡ… làm theo cửa sổ của trình gỡ (có thể Windows sẽ hỏi quyền).</p>}
      {after && (
        <LeftoverPanel key={after.app.id} app={after.app} stillInstalled={after.stillInstalled} onClose={() => setAfter(null)} />
      )}
      <ul className="app-list">
        {rows.map((a) => (
          <AppRow key={a.id} app={a} busy={uninstalling != null} onUninstall={() => uninstall(a)} />
        ))}
      </ul>
    </div>
  );
}
