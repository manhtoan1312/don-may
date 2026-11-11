import { useMemo, useState } from "react";
import type { ProcInfo } from "./api";
import { AiPanel } from "./AiPanel";
import { ProcessRow } from "./ProcessRow";
import { StatsBar } from "./StatsBar";
import { useSnapshot } from "./useSnapshot";

type SortKey = "cpu" | "memory" | "name";
const SORTERS: Record<SortKey, (a: ProcInfo, b: ProcInfo) => number> = {
  cpu: (a, b) => b.cpu - a.cpu,
  memory: (a, b) => b.memory - a.memory,
  name: (a, b) => a.name.localeCompare(b.name),
};

export function ProcessesView() {
  const { data, error, refresh } = useSnapshot();
  const [query, setQuery] = useState("");
  const [onlyFlagged, setOnlyFlagged] = useState(false);
  const [sort, setSort] = useState<SortKey>("cpu");
  const [selectedPid, setSelectedPid] = useState<number | null>(null);

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase();
    return (data?.processes ?? [])
      .filter((p) => (!onlyFlagged || p.flags.length) && (!q || p.name.toLowerCase().includes(q)))
      .sort(SORTERS[sort]);
  }, [data, query, onlyFlagged, sort]);

  if (error) return <p className="error">Lỗi đọc tiến trình: {error}</p>;
  if (!data) return <p className="muted">Đang đọc thông tin hệ thống…</p>;

  const selected = data.processes.find((p) => p.pid === selectedPid);
  const header = (key: SortKey, label: string, cls = "") => (
    <th className={`${cls} sortable ${sort === key ? "active" : ""}`} onClick={() => setSort(key)}>
      {label}
    </th>
  );

  return (
    <div className="processes">
      <StatsBar data={data} />
      <div className="toolbar">
        <input placeholder="Tìm tiến trình…" value={query} onChange={(e) => setQuery(e.target.value)} />
        <label>
          <input type="checkbox" checked={onlyFlagged} onChange={(e) => setOnlyFlagged(e.target.checked)} />
          Chỉ hiện tiến trình đáng ngờ
        </label>
      </div>
      <div className="content">
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                {header("name", "Tên")}
                <th className="num">PID</th>
                {header("cpu", "CPU", "num")}
                {header("memory", "RAM", "num")}
                <th />
              </tr>
            </thead>
            <tbody>
              {rows.map((p) => (
                <ProcessRow
                  key={p.pid}
                  proc={p}
                  selected={p.pid === selectedPid}
                  onSelect={() => setSelectedPid(p.pid)}
                  onKilled={refresh}
                />
              ))}
            </tbody>
          </table>
        </div>
        {selected && (
          <AiPanel key={selected.pid} proc={selected} onClose={() => setSelectedPid(null)} />
        )}
      </div>
    </div>
  );
}
