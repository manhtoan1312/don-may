import { useCallback, useEffect, useState } from "react";
import { getSnapshot, type Snapshot } from "./api";

export function useSnapshot(intervalMs = 2000) {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(
    () =>
      getSnapshot()
        .then((d) => {
          setData(d);
          setError(null);
        })
        .catch((e) => setError(String(e))),
    [],
  );

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, intervalMs);
    return () => clearInterval(id);
  }, [refresh, intervalMs]);

  return { data, error, refresh };
}
