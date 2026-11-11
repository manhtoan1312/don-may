import { useState } from "react";

type Props = { onConfirm: () => Promise<void>; blocked?: string | null; onDone?: () => void };

/** Nút "Vào Thùng rác" bấm 2 lần. Backend luôn kiểm tra lại quy tắc trước khi xóa. */
export function TrashButton({ onConfirm, blocked, onDone }: Props) {
  const [state, setState] = useState<"idle" | "confirm" | "busy" | "done">("idle");
  const [error, setError] = useState<string | null>(null);

  if (blocked) return <small className="muted" title={blocked}>Giữ lại: {blocked}</small>;
  if (state === "done") return <small className="success">Đã chuyển vào Thùng rác</small>;

  const click = async () => {
    if (state === "idle") return setState("confirm");
    setState("busy");
    try {
      await onConfirm();
      setState("done");
      onDone?.();
    } catch (e) {
      setError(String(e));
      setState("idle");
    }
  };

  return (
    <span>
      <button className={state === "confirm" ? "danger" : ""} onClick={click} disabled={state === "busy"}
        onBlur={() => state === "confirm" && setState("idle")}>
        {state === "confirm" ? "Chắc chắn?" : state === "busy" ? "Đang xóa…" : "Vào Thùng rác"}
      </button>
      {error && <small className="error"> {error}</small>}
    </span>
  );
}
