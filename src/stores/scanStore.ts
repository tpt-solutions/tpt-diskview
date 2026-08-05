import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import type { FileNode, ScanProgress, ViewMode } from "../types";

interface ScanState {
  scanPath: string;
  isScanning: boolean;
  tree: FileNode | null;
  selectedNode: FileNode | null;
  breadcrumbs: FileNode[];
  progress: ScanProgress | null;
  error: string | null;
  viewMode: ViewMode;
  sunburstCenter: FileNode | null;
  sunburstRadius: number;
  showSettings: boolean;
  followSymlinks: boolean;
  maxDepth: number | null;
  excludePatterns: string[];
}

const [state, setState] = createStore<ScanState>({
  scanPath: "C:\\",
  isScanning: false,
  tree: null,
  selectedNode: null,
  breadcrumbs: [],
  progress: null,
  error: null,
  viewMode: "tree",
  sunburstCenter: null,
  sunburstRadius: 250,
  showSettings: false,
  followSymlinks: false,
  maxDepth: null,
  excludePatterns: [],
});

export { state, setState };

export const startScan = async () => {
  try {
    setState({ isScanning: true, tree: null, selectedNode: null, breadcrumbs: [], error: null, progress: null });

    await invoke("start_scan", {
      options: {
        root: state.scanPath,
        follow_symlinks: state.followSymlinks,
        max_depth: state.maxDepth,
        exclude_patterns: state.excludePatterns,
      },
    });

    const poll = setInterval(async () => {
      try {
        const status: any = await invoke("scan_status");
        if (status.result) {
          setState({ tree: status.result.root, isScanning: false });
          clearInterval(poll);
        } else if (status.error) {
          setState({ error: status.error, isScanning: false });
          clearInterval(poll);
        } else {
          if (status.partial_tree) {
            setState({ tree: status.partial_tree.root });
          }
          setState({
            progress: {
              path: status.current_path,
              files_scanned: status.files_scanned,
              bytes_scanned: status.bytes_scanned,
            },
          });
        }
      } catch (e) {
        clearInterval(poll);
        setState({ isScanning: false });
      }
    }, 100);
  } catch (e) {
    setState({ error: String(e), isScanning: false });
  }
};

export const cancelScan = async () => {
  try {
    await invoke("cancel_scan");
    setState({ isScanning: false, progress: null });
  } catch (e) {
    setState({ error: String(e) });
  }
};

export const navigateTo = (node: FileNode) => {
  setState({ selectedNode: node, breadcrumbs: [...state.breadcrumbs, node] });
};

export const goBackTo = (index: number) => {
  setState({
    breadcrumbs: state.breadcrumbs.slice(0, index + 1),
    selectedNode: state.breadcrumbs[index],
  });
};

export const goUp = () => {
  if (state.breadcrumbs.length > 1) {
    const newBreadcrumbs = state.breadcrumbs.slice(0, -1);
    setState({
      breadcrumbs: newBreadcrumbs,
      selectedNode: newBreadcrumbs[newBreadcrumbs.length - 1],
    });
  } else {
    setState({
      breadcrumbs: [],
      selectedNode: state.tree,
    });
  }
};

export const setViewMode = (mode: ViewMode) => setState({ viewMode: mode });

export const setScanPath = (path: string) => setState({ scanPath: path });

export const setError = (err: string | null) => setState({ error: err });

export const deselectNode = () => {
  setState({ selectedNode: null, breadcrumbs: [] });
};
