import { useState, type ComponentType } from "react";
import "./App.css";
import { AppsView } from "./features/apps/AppsView";
import { FilesView } from "./features/files/FilesView";
import { CleanerView } from "./features/cleaner/CleanerView";
import { ProcessesView } from "./features/processes/ProcessesView";
import { SettingsView } from "./features/settings/SettingsView";

const TABS = {
  processes: { label: "Tiến trình", View: ProcessesView },
  cleaner: { label: "Dọn rác", View: CleanerView },
  files: { label: "Thống kê file", View: FilesView },
  apps: { label: "Gỡ app", View: AppsView },
  settings: { label: "Cài đặt", View: SettingsView },
} satisfies Record<string, { label: string; View: ComponentType }>;
type Tab = keyof typeof TABS;

export default function App() {
  const [tab, setTab] = useState<Tab>("processes");
  const { View } = TABS[tab];
  return (
    <main className="app">
      <nav className="tabs">
        <h1>Dọn Máy</h1>
        {(Object.keys(TABS) as Tab[]).map((t) => (
          <button key={t} className={tab === t ? "active" : ""} onClick={() => setTab(t)}>
            {TABS[t].label}
          </button>
        ))}
      </nav>
      <View />
    </main>
  );
}
