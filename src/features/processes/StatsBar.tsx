import type { Snapshot } from "./api";
import { formatBytes, formatPercent } from "../../lib/format";

function Meter({ label, value, detail }: { label: string; value: number; detail: string }) {
  const level = value >= 90 ? "danger" : value >= 75 ? "warn" : "ok";
  return (
    <div className="meter">
      <div className="meter-head">
        <span>{label}</span>
        <strong>{detail}</strong>
      </div>
      <div className="meter-track" role="progressbar" aria-label={label} aria-valuenow={Math.round(value)}>
        <div className={`meter-fill ${level}`} style={{ width: `${Math.min(value, 100)}%` }} />
      </div>
    </div>
  );
}

export function StatsBar({ data }: { data: Snapshot }) {
  const memPct = (data.memUsed / data.memTotal) * 100;
  const flagged = data.processes.filter((p) => p.flags.length).length;
  return (
    <section className="stats">
      <Meter label="CPU" value={data.cpu} detail={formatPercent(data.cpu)} />
      <Meter
        label="RAM"
        value={memPct}
        detail={`${formatBytes(data.memUsed)} / ${formatBytes(data.memTotal)}`}
      />
      <div className="stat-box">
        <span>Tiến trình</span>
        <strong>{data.processes.length}</strong>
      </div>
      <div className={`stat-box ${flagged ? "warn-text" : ""}`}>
        <span>Đáng ngờ</span>
        <strong>{flagged}</strong>
      </div>
    </section>
  );
}
