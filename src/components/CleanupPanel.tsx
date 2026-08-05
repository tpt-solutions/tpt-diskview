import { Show, For } from "solid-js";
import { state, toggleCandidate, executeCleanup, setCleanupMode, setShowCleanup, resetCleanup, loadCandidates } from "../stores/cleanupStore";
import { state as scanState } from "../stores/scanStore";
import { formatSize } from "../utils/format";

export default function CleanupPanel() {
  const selectedSize = () => state.candidates
    .filter(c => state.selectedForCleanup.includes(c.path))
    .reduce((sum, c) => sum + c.size, 0);

  return (
    <Show when={state.showCleanup}>
      <div class="cleanup-panel">
        <div class="cleanup-header">
          <h2>Cleanup Candidates</h2>
          <button onClick={() => setShowCleanup(false)}>×</button>
        </div>

        <Show when={state.mode === "preview"}>
          <div class="cleanup-mode-bar">
            <span>Preview Mode - Select items to clean up</span>
            <button class="primary" onClick={() => setCleanupMode("confirm")}>
              Review Selection ({state.selectedForCleanup.length} items)
            </button>
          </div>
          <div class="cleanup-list">
            <For each={state.candidates}>
              {(candidate) => {
                const isSelected = state.selectedForCleanup.includes(candidate.path);
                return (
                  <div class="cleanup-item" style={{ opacity: isSelected ? 1 : 0.6 }}>
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => toggleCandidate(candidate.path)}
                    />
                    <span class="category">[{candidate.category}]</span>
                    <span class="path">{candidate.path}</span>
                    <span class="size">{formatSize(candidate.size)}</span>
                    <span class="description">{candidate.description}</span>
                  </div>
                );
              }}
            </For>
          </div>
          <div class="cleanup-summary">
            Selected: {formatSize(selectedSize())}
          </div>
        </Show>

        <Show when={state.mode === "confirm"}>
          <div class="cleanup-mode-bar warning">
            <span>⚠ Confirm Deletion - This will move {state.selectedForCleanup.length} items to Recycle Bin</span>
          </div>
          <div class="cleanup-list">
            <For each={state.candidates.filter(c => state.selectedForCleanup.includes(c.path))}>
              {(candidate) => (
                <div class="cleanup-item">
                  <span class="category">[{candidate.category}]</span>
                  <span class="path">{candidate.path}</span>
                  <span class="size">{formatSize(candidate.size)}</span>
                  <span class="description">{candidate.description}</span>
                </div>
              )}
            </For>
          </div>
          <div class="cleanup-actions">
            <button onClick={() => setCleanupMode("preview")}>Back</button>
            <button class="danger" onClick={executeCleanup}>
              Move to Recycle Bin ({formatSize(selectedSize())})
            </button>
          </div>
        </Show>

        <Show when={state.mode === "complete" && state.result}>
          <div class="cleanup-mode-bar success">
            <span>✓ Cleanup Complete - Removed {state.result!.removed} items, freed {formatSize(state.result!.freed)}</span>
          </div>
          <div class="cleanup-actions">
            <button onClick={() => {
              resetCleanup();
              loadCandidates(scanState.scanPath);
            }}>
              Done
            </button>
          </div>
        </Show>
      </div>
    </Show>
  );
}
