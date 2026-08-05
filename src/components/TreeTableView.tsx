import { Show } from "solid-js";
import { state, navigateTo } from "../stores/scanStore";
import TreeView from "./TreeView";

export default function TreeTableView() {
  return (
    <Show when={state.viewMode === "tree" && state.tree}>
      <div class="table-header">
        <span class="name">Name</span>
        <span class="size">Size</span>
        <span class="percentage">%</span>
        <span class="count">Items</span>
      </div>
      <div class="table-view">
        <TreeView
          node={state.selectedNode || state.tree!}
          rootSize={state.tree!.size}
          depth={0}
          onNavigate={navigateTo}
        />
      </div>
    </Show>
  );
}
