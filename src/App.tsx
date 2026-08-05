import { onMount, onCleanup } from "solid-js";
import "./styles.css";

import Header from "./components/Header";
import ScanProgress from "./components/ScanProgress";
import ErrorBar from "./components/ErrorBar";
import Sidebar from "./components/Sidebar";
import EmptyState from "./components/EmptyState";
import TreemapView from "./components/TreemapView";
import SunburstView from "./components/SunburstView";
import TreeTableView from "./components/TreeTableView";
import CleanupPanel from "./components/CleanupPanel";
import DetailsPanel from "./components/DetailsPanel";
import Settings from "./components/Settings";
import { deselectNode } from "./stores/scanStore";

function App() {
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        deselectNode();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  return (
    <div class="app">
      <Header />
      <ErrorBar />
      <ScanProgress />
      <div class="main-content">
        <Sidebar />
        <main class="visualization">
          <EmptyState />
          <TreemapView />
          <SunburstView />
          <TreeTableView />
        </main>
      </div>
      <CleanupPanel />
      <DetailsPanel />
      <Settings />
    </div>
  );
}

export default App;
