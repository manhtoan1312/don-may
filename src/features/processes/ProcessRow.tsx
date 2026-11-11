import { useState, type MouseEvent } from "react";
import { killProcess, type ProcInfo } from "./api";
import { formatBytes, formatPercent } from "../../lib/format";

type Props = {
  proc: ProcInfo;
  selected: boolean;
  onSelect: () => void;
  onKilled: () => void;
};

export function ProcessRow({ proc, selected, onSelect, onKilled }: Props) {
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const kill = async (e: MouseEvent) => {
    e.stopPropagation();
    if (!confirming) return setConfirming(true);
    try {
      await killProcess(proc.pid);
      onKilled();
    } catch (err) {
      setError(String(err));
    }
    setConfirming(false);
  };

  return (
    <tr
      className={`${proc.flags.length ? "flagged" : ""} ${selected ? "selected" : ""}`}
      onClick={onSelect}
    >
      <td title={proc.exe ?? ""}>
        {proc.flags.length > 0 && <span className="flag-dot" title={proc.flags.join("\n")}>⚠</span>}
        {proc.name}
        {error && <div className="row-error">{error}</div>}
      </td>
      <td className="num">{proc.pid}</td>
      <td className="num">{formatPercent(proc.cpu)}</td>
      <td className="num">{formatBytes(proc.memory)}</td>
      <td className="actions">
        <button className={confirming ? "danger" : ""} onClick={kill} onBlur={() => setConfirming(false)}>
          {confirming ? "Chắc chắn?" : "Dừng"}
        </button>
      </td>
    </tr>
  );
}
