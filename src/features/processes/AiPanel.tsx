import { useState } from "react";
import { analyzeProcess, type ProcInfo, type Risk, type Verdict } from "./api";

const RISK_LABEL: Record<Risk, string> = { thap: "Thấp", trung_binh: "Trung bình", cao: "Cao" };

type Props = { proc: ProcInfo; onClose: () => void };

export function AiPanel({ proc, onClose }: Props) {
  const [verdict, setVerdict] = useState<Verdict | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const ask = async () => {
    setLoading(true);
    setError(null);
    try {
      setVerdict(await analyzeProcess(proc.pid));
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  return (
    <aside className="ai-panel">
      <header>
        <h2>{proc.name}</h2>
        <button onClick={onClose} aria-label="Đóng">✕</button>
      </header>
      <div className="muted">{proc.exe ?? "Không đọc được đường dẫn"}</div>
      {proc.flags.length > 0 && (
        <ul className="flag-list">
          {proc.flags.map((f) => <li key={f}>⚠ {f}</li>)}
        </ul>
      )}
      <button className="primary" onClick={ask} disabled={loading}>
        {loading ? "AI đang phân tích…" : "Phân tích bằng AI (Ollama, chạy trên máy)"}
      </button>
      {error && <p className="error">{error}</p>}
      {verdict && (
        <div className="ai-answer">
          <p>
            <span className={`risk risk-${verdict.risk}`}>Mức nguy hiểm: {RISK_LABEL[verdict.risk]}</span>
            {verdict.risk !== verdict.aiRisk && (
              <small className="muted"> (AI đánh giá {RISK_LABEL[verdict.aiRisk]}, quy tắc của app đã nâng mức)</small>
            )}
          </p>
          <p><strong>Chương trình:</strong> {verdict.program}</p>
          <ul>{verdict.reasons.map((r) => <li key={r}>{r}</li>)}</ul>
          <p><strong>Nên làm:</strong> {verdict.advice}</p>
          <details>
            <summary>Dữ liệu AI đã thấy</summary>
            <pre className="proc-info">{verdict.info}</pre>
          </details>
          <small className="muted">AI có thể sai. App không tự dừng tiến trình.</small>
        </div>
      )}
    </aside>
  );
}
