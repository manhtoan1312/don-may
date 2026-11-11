import { useEffect, useState, type FormEvent } from "react";
import { getSettings, saveSettings, testNotification, type Settings } from "./api";

type NumberKey = "cpuThreshold" | "ramThreshold" | "sustainSecs" | "cooldownMins";

const NUMBER_FIELDS: { key: NumberKey; label: string; hint: string; min: number; max: number }[] = [
  { key: "cpuThreshold", label: "Ngưỡng CPU (%)", hint: "10–100", min: 10, max: 100 },
  { key: "ramThreshold", label: "Ngưỡng RAM (%)", hint: "10–100", min: 10, max: 100 },
  { key: "sustainSecs", label: "Vượt ngưỡng liên tục (giây)", hint: "Tránh báo khi chỉ nhảy vọt vài giây", min: 4, max: 600 },
  { key: "cooldownMins", label: "Không báo lại trong (phút)", hint: "1–240", min: 1, max: 240 },
];

export function SettingsView() {
  const [form, setForm] = useState<Settings | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    getSettings().then(setForm).catch((e) => setStatus(String(e)));
  }, []);

  if (!form) return <p className="muted">{status ?? "Đang tải cài đặt…"}</p>;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) => setForm({ ...form, [key]: value });

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    try {
      setForm(await saveSettings(form));
      setStatus("Đã lưu.");
    } catch (err) {
      setStatus(String(err));
    }
  };

  return (
    <form className="settings" onSubmit={submit}>
      <h2>Cảnh báo CPU / RAM</h2>
      <label className="check">
        <input type="checkbox" checked={form.alertsEnabled} onChange={(e) => set("alertsEnabled", e.target.checked)} />
        Bật cảnh báo (thông báo Windows)
      </label>
      {NUMBER_FIELDS.map((f) => (
        <label key={f.key} className="field">
          <span>{f.label}</span>
          <input
            type="number"
            min={f.min}
            max={f.max}
            value={form[f.key]}
            onChange={(e) => set(f.key, Number(e.target.value))}
          />
          <small className="muted">{f.hint}</small>
        </label>
      ))}

      <h2>AI (Ollama)</h2>
      <label className="field">
        <span>Tên model</span>
        <input value={form.ollamaModel} onChange={(e) => set("ollamaModel", e.target.value)} />
        <small className="muted">Ví dụ: gemma4:e4b — xem danh sách bằng lệnh `ollama list`</small>
      </label>

      <div className="settings-actions">
        <button type="submit" className="primary">Lưu</button>
        <button type="button" onClick={() => testNotification().catch((e) => setStatus(String(e)))}>
          Gửi thông báo thử
        </button>
        {status && <span className="muted">{status}</span>}
      </div>
    </form>
  );
}
