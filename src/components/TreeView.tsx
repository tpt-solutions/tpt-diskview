import { Show, For } from "solid-js";
import type { FileNode } from "../types";
import { formatSize, getFileTypeColor } from "../utils/format";

interface TreeViewProps {
  node: FileNode;
  rootSize: number;
  depth: number;
  onNavigate: (node: FileNode) => void;
}

export default function TreeView(props: TreeViewProps) {
  const percentage = () => ((props.node.size / props.rootSize) * 100).toFixed(1);

  return (
    <>
      <div style={{ "padding-left": `${props.depth * 20}px` }}>
        <div
          class="tree-row"
          style={{ cursor: "pointer" }}
          onClick={() => props.onNavigate(props.node)}
        >
          <span style={{ color: getFileTypeColor(props.node.node_type), "margin-right": "8px" }}>
            {props.node.node_type === "Directory" ? "\uD83D\uDCC1" : "\uD83D\uDCC4"}
          </span>
          <span class="name">{props.node.name}</span>
          <span class="size">{formatSize(props.node.size)}</span>
          <span class="percentage">{percentage()}%</span>
          <span class="count">{props.node.item_count} items</span>
        </div>
      </div>
      <Show when={props.node.node_type === "Directory" && props.node.children.length > 0}>
        <For each={props.node.children.slice(0, 50)}>
          {(child) => (
            <TreeView node={child} rootSize={props.rootSize} depth={props.depth + 1} onNavigate={props.onNavigate} />
          )}
        </For>
      </Show>
    </>
  );
}
