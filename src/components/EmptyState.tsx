import { Show } from "solid-js";
import { state } from "../stores/scanStore";

export default function EmptyState() {
  return (
    <>
      <Show when={!state.tree && !state.isScanning}>
        <div class="empty-state">
          <h2>No scan data</h2>
          <p>Enter a path and click Start Scan.</p>
        </div>
      </Show>
      <Show when={!state.tree && state.isScanning && !state.progress}>
        <div class="empty-state">
          <span class="loading-indicator" />
          <p>Starting scan...</p>
        </div>
      </Show>
    </>
  );
}
