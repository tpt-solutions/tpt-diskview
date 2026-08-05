import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import type { CleanupCandidate } from "../types";

interface CleanupState {
  candidates: CleanupCandidate[];
  showCleanup: boolean;
  mode: "preview" | "confirm" | "complete";
  selectedForCleanup: string[];
  result: { removed: number; freed: number } | null;
}

const [state, setState] = createStore<CleanupState>({
  candidates: [],
  showCleanup: false,
  mode: "preview",
  selectedForCleanup: [],
  result: null,
});

export { state };

export const loadCandidates = async (path: string) => {
  try {
    const candidates = await invoke<CleanupCandidate[]>("detect_cleanup_candidates", { path });
    setState({ candidates, showCleanup: true });
  } catch (e) {
    console.error(e);
  }
};

export const toggleCandidate = (path: string) => {
  const idx = state.selectedForCleanup.indexOf(path);
  if (idx >= 0) {
    setState("selectedForCleanup", (prev) => prev.filter((p) => p !== path));
  } else {
    setState("selectedForCleanup", (prev) => [...prev, path]);
  }
};

export const executeCleanup = async () => {
  try {
    const result = await invoke<{ removed: number; freed: number }>("cleanup_selected", {
      paths: state.selectedForCleanup,
    });
    setState({ result, mode: "complete" });
  } catch (e) {
    console.error(e);
  }
};

export const setCleanupMode = (mode: "preview" | "confirm" | "complete") => setState({ mode });

export const setShowCleanup = (show: boolean) => setState({ showCleanup: show });

export const resetCleanup = () => {
  setState({
    mode: "preview",
    selectedForCleanup: [],
    result: null,
  });
};
