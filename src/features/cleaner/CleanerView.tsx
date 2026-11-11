import { AppDataPanel } from "./AppDataPanel";
import { JunkPanel } from "./JunkPanel";

export function CleanerView() {
  return (
    <div className="cleaner">
      <JunkPanel />
      <AppDataPanel />
    </div>
  );
}
