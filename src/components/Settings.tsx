import { Show } from "solid-js";
import { state, setState, setScanPath } from "../stores/scanStore";

export default function Settings() {
  const handleSave = () => {
    setState("showSettings", false);
  };

  return (
    <Show when={state.showSettings}>
      <div class="settings-overlay" onClick={() => setState("showSettings", false)}>
        <div class="settings-panel" onClick={(e) => e.stopPropagation()}>
          <div class="settings-header">
            <h2>Settings</h2>
            <button onClick={() => setState("showSettings", false)}>×</button>
          </div>
          <div class="settings-body">
            <div class="setting-row">
              <label for="default-path">Default scan path</label>
              <input
                id="default-path"
                type="text"
                value={state.scanPath}
                onInput={(e) => setScanPath(e.currentTarget.value)}
              />
            </div>
            <div class="setting-row">
              <label for="follow-symlinks">
                <input
                  id="follow-symlinks"
                  type="checkbox"
                  checked={state.followSymlinks}
                  onChange={(e) => setState("followSymlinks", e.currentTarget.checked)}
                />
                Follow symbolic links
              </label>
            </div>
            <div class="setting-row">
              <label for="max-depth">Max scan depth (leave empty for unlimited)</label>
              <input
                id="max-depth"
                type="number"
                value={state.maxDepth ?? ""}
                onInput={(e) => {
                  const val = e.currentTarget.value;
                  setState("maxDepth", val ? parseInt(val) : null);
                }}
                placeholder="Unlimited"
                min="1"
              />
            </div>
            <div class="setting-row">
              <label for="exclude-patterns">Exclude patterns (one per line)</label>
              <textarea
                id="exclude-patterns"
                rows={4}
                value={state.excludePatterns.join("\n")}
                onInput={(e) => setState("excludePatterns", e.currentTarget.value.split("\n").filter(p => p.trim()))}
                placeholder="*.log&#10;node_modules&#10;.git"
              />
            </div>
          </div>
          <div class="settings-actions">
            <button onClick={() => setState("showSettings", false)}>Cancel</button>
            <button class="primary" onClick={handleSave}>Save</button>
          </div>
        </div>
      </div>
    </Show>
  );
}
