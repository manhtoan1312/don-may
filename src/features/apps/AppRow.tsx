import { useState } from "react";
import { formatBytes, formatInstallDate } from "../../lib/format";
import { explainApp, type Advice, type App, type AppVerdict } from "./api";

const ADVICE_LABEL: Record<Advice, string> = {
  giu: "Nên giữ",
  co_the_go: "Có thể gỡ",
  nen_go: "Nên gỡ",
  khong_ro: "Không rõ",
};

type Props = { app: App; busy: boolean; onUninstall: () => void };

export function AppRow({ app, busy, onUninstall }: Props) {
  const [confirming, setConfirming] = useState(false);
  const [verdict, setVerdict] = useState<AppVerdict | null>(null);
  const [aiState, setAiState] = useState<"idle" | "loading" | string>("idle");

  const ask = async () => {
    setAiState("loading");
    try {
      setVerdict(await explainApp(app.id));
      setAiState("idle");
    } catch (e) {
      setAiState(String(e));
    }
  };

  const uninstall = () => {
    if (!confirming) return setConfirming(true);
    setConfirming(false);
    onUninstall();
  };

  return (
    <li className="app-row">
      <div className="app-main">
        <div>
          <strong>{app.name}</strong>
          <div className="muted small">
            {[app.publisher, app.version, formatInstallDate(app.installDate)].filter(Boolean).join(" · ")}
          </div>
        </div>
        <span className="num">{app.sizeKb ? formatBytes(app.sizeKb * 1024) : "—"}</span>
        <button onClick={ask} disabled={aiState === "loading"}>{aiState === "loading" ? "Đang hỏi…" : "Hỏi AI"}</button>
        <button className={confirming ? "danger" : ""} onClick={uninstall} onBlur={() => setConfirming(false)} disabled={busy}>
          {confirming ? "Chắc chắn gỡ?" : "Gỡ"}
        </button>
      </div>
      {aiState !== "idle" && aiState !== "loading" && <small className="error">{aiState}</small>}
      {verdict && (
        <div className="folder-verdict">
          <span className={`advice advice-${verdict.advice}`}>{ADVICE_LABEL[verdict.advice]}</span>{" "}
          {verdict.purpose}. {verdict.reason}
          {verdict.overridden && <small className="muted"> (quy tắc của app đã chuyển sang "Nên giữ")</small>}
        </div>
      )}
    </li>
  );
}
