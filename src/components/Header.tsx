import { Show } from "solid-js";
import { state, startScan, cancelScan, setScanPath, setState } from "../stores/scanStore";
import { loadCandidates } from "../stores/cleanupStore";

export default function Header() {
  return (
    <header>
      <h1>tpt-diskview</h1>
      <div class="controls">
        <input
          type="text"
          value={state.scanPath}
          onInput={(e) => setScanPath(e.currentTarget.value)}
          placeholder="Path to scan..."
        />
        <button onClick={startScan} disabled={state.isScanning}>
          {state.isScanning ? "Scanning..." : "Start Scan"}
        </button>
        <Show when={state.isScanning}>
          <button onClick={cancelScan} class="danger">
            Cancel Scan
          </button>
        </Show>
        <button onClick={() => loadCandidates(state.scanPath)}>
          Detect Cleanup
        </button>
      </div>
      <div class="header-actions">
        <button class="icon-btn" onClick={() => setState("showSettings", true)} title="Settings">
          ⚙
        </button>
      </div>
    </header>
  );
}
