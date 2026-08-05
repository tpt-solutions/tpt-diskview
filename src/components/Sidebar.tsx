import { Show, For } from "solid-js";
import { state, setViewMode, navigateTo, goBackTo } from "../stores/scanStore";
import TreeView from "./TreeView";

export default function Sidebar() {
  const currentMode = () => state.viewMode;

  return (
    <aside class="sidebar">
      <div class="view-tabs">
        <button
          class={currentMode() === "tree" ? "active" : ""}
          onClick={() => setViewMode("tree")}
        >
          Tree
        </button>
        <button
          class={currentMode() === "treemap" ? "active" : ""}
          onClick={() => setViewMode("treemap")}
        >
          Treemap
        </button>
        <button
          class={currentMode() === "sunburst" ? "active" : ""}
          onClick={() => setViewMode("sunburst")}
        >
          Sunburst
        </button>
      </div>

      <Show when={state.breadcrumbs.length > 0}>
        <div class="breadcrumbs">
          <For each={state.breadcrumbs}>
            {(crumb, i) => (
              <span
                class="breadcrumb"
                onClick={() => goBackTo(i())}
              >
                {crumb.name} /
              </span>
            )}
          </For>
        </div>
      </Show>

      <Show when={state.tree}>
        <div class="tree-view">
          <TreeView
            node={state.selectedNode || state.tree!}
            rootSize={state.tree!.size}
            depth={0}
            onNavigate={navigateTo}
          />
        </div>
      </Show>
    </aside>
  );
}
