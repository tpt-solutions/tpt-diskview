import { Show } from "solid-js";
import { state } from "../stores/scanStore";
import { formatSize } from "../utils/format";

export default function DetailsPanel() {
  return (
    <Show when={state.selectedNode}>
      <div class="details-panel">
        <h3>{state.selectedNode!.name}</h3>
        <p>Path: {state.selectedNode!.path}</p>
        <p>Size: {formatSize(state.selectedNode!.size)}</p>
        <p>Items: {state.selectedNode!.item_count}</p>
        <p>Type: {state.selectedNode!.node_type}</p>
      </div>
    </Show>
  );
}
