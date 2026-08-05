import { Show } from "solid-js";
import type { JSX } from "solid-js";
import { state, navigateTo, goUp } from "../stores/scanStore";
import { getFileTypeColor } from "../utils/format";

export default function SunburstView() {
  return (
    <Show when={state.viewMode === "sunburst" && state.tree}>
      <svg width="100%" height="100%" viewBox="0 0 800 600">
        {(() => {
          const node = state.selectedNode || state.tree!;
          if (!node.children.length) return null;

          const total = node.children.reduce((sum, c) => sum + c.size, 0);
          if (total === 0) return null;

          const cx = 400;
          const cy = 300;
          const r = 250;
          const isRoot = state.breadcrumbs.length === 0;
          const elements: JSX.Element[] = [];

          if (!isRoot) {
            elements.push(
              <circle
                cx={cx}
                cy={cy}
                r={r * 0.6}
                fill="#16213e"
                stroke="#4fc3f7"
                stroke-width={2}
                opacity={0.9}
                style={{ cursor: "pointer" }}
                onClick={goUp}
              />
            );
          }

          let angle = 0;
          for (const child of node.children) {
            const ratio = child.size / total;
            const childAngle = (2 * Math.PI) * ratio;

            if (childAngle > 0.01) {
              const innerR = r * 0.6;
              const outerR = r;
              const x1 = cx + innerR * Math.cos(angle);
              const y1 = cy + innerR * Math.sin(angle);
              const x2 = cx + outerR * Math.cos(angle);
              const y2 = cy + outerR * Math.sin(angle);
              const x3 = cx + outerR * Math.cos(angle + childAngle);
              const y3 = cy + outerR * Math.sin(angle + childAngle);
              const x4 = cx + innerR * Math.cos(angle + childAngle);
              const y4 = cy + innerR * Math.sin(angle + childAngle);

              const largeArc = childAngle > Math.PI ? 1 : 0;

              const d = `M ${x1} ${y1} L ${x2} ${y2} A ${outerR} ${outerR} 0 ${largeArc} 1 ${x3} ${y3} L ${x4} ${y4} A ${innerR} ${innerR} 0 ${largeArc} 0 ${x1} ${y1}`;

              elements.push(
                <path
                  d={d}
                  fill={getFileTypeColor(child.node_type)}
                  stroke="#1a1a2e"
                  stroke-width={1}
                  opacity={0.8}
                  style={{ cursor: "pointer" }}
                  onClick={() => {
                    if (child.node_type === "Directory" && child.children.length > 0) {
                      // Drill-down into directory (preserved behavior)
                    } else {
                      navigateTo(child);
                    }
                  }}
                />
              );
            }

            angle += childAngle;
          }

          return elements;
        })()}
      </svg>
    </Show>
  );
}
