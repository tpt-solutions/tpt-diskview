import { Show } from "solid-js";
import { state, navigateTo } from "../stores/scanStore";
import { getFileTypeColor } from "../utils/format";
import { squarify } from "../utils/treemap";

export default function TreemapView() {
  return (
    <Show when={state.viewMode === "treemap" && state.tree}>
      <svg width="100%" height="100%" viewBox="0 0 800 600">
        {(() => {
          const node = state.selectedNode || state.tree!;
          if (!node.children.length) return null;
          return squarify(node.children, 800, 600, 0, 0).map(({ x, y, w, h, node: child }) => (
            <rect
              x={x}
              y={y}
              width={Math.max(1, w - 1)}
              height={Math.max(1, h - 1)}
              fill={getFileTypeColor(child.node_type)}
              stroke="#1a1a2e"
              stroke-width={1}
              opacity={0.8}
              style={{ cursor: "pointer" }}
              onClick={() => navigateTo(child)}
            />
          ));
        })()}
      </svg>
    </Show>
  );
}
