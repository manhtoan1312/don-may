import { useState } from "react";
import { TrashButton } from "../../components/TrashButton";
import { formatBytes } from "../../lib/format";
import { deleteFile, explainFile, VERDICT_LABEL, type BigFile, type FileVerdict } from "./api";

const AI_LABEL = { co: "AI: có thể xóa", khong: "AI: không nên xóa", khong_ro: "AI: không rõ" } as const;

function BigFileRow({ file }: { file: BigFile }) {
  const [ai, setAi] = useState<FileVerdict | null>(null);
  const [state, setState] = useState<"idle" | "loading" | string>("idle");
  const locked = file.verdict === "khong_duoc_xoa";

  const ask = async () => {
    setState("loading");
    try {
      setAi(await explainFile(file.path));
      setState("idle");
    } catch (e) {
      setState(String(e));
    }
  };

  return (
    <li className="app-row">
      <div className="file-main">
        <span className="path" title={file.path}>{file.path}</span>
        <strong className="num">{formatBytes(file.bytes)}</strong>
        <small className="muted num">{file.ageDays} ngày</small>
        <button onClick={ask} disabled={state === "loading"}>{state === "loading" ? "Đang hỏi…" : "Hỏi AI"}</button>
        <TrashButton onConfirm={() => deleteFile(file.path)} blocked={locked ? "file hệ thống" : null} />
      </div>
      <div className="folder-verdict">
        <span className={`verdict verdict-${file.verdict}`}>{VERDICT_LABEL[file.verdict]}</span> {file.reason}
      </div>
      {state !== "idle" && state !== "loading" && <small className="error">{state}</small>}
      {ai && (
        <div className="folder-verdict">
          <span className={`safe safe-${ai.safeToDelete}`}>{AI_LABEL[ai.safeToDelete]}</span>{" "}
          <strong>{ai.what}</strong>: {ai.reason}
        </div>
      )}
    </li>
  );
}

export function BigFiles({ files }: { files: BigFile[] }) {
  return (
    <section className="card">
      <h2>50 file lớn nhất</h2>
      <small className="muted">
        Nhãn đánh giá do quy tắc của app đưa ra; AI chỉ giải thích thêm. Xóa = chuyển vào Thùng rác.
      </small>
      <ul className="app-list">
        {files.map((f) => <BigFileRow key={f.path} file={f} />)}
      </ul>
    </section>
  );
}
