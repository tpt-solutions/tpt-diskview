import { createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

interface FileNode {
  name: string;
  path: string;
  size: number;
  allocated: number;
  node_type: "Directory" | "File" | "Symlink" | "Junction";
  children: FileNode[];
  item_count: number;
  last_modified: number | null;
}

interface ScanProgress {
  path: string;
  files_scanned: number;
  bytes_scanned: number;
}

interface PartialTree {
  root: FileNode;
}

interface CleanupCandidate {
  path: string;
  size: number;
  category: "TempFiles" | "Duplicates" | "DockerVolumes" | "BrowserCache";
  description: string;
}

type ViewMode = "treemap" | "sunburst" | "tree";

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function getFileTypeColor(type: string): string {
  switch (type) {
    case "Directory": return "#4fc3f7";
    case "File": return "#81c784";
    case "Symlink": return "#ffb74d";
    case "Junction": return "#ce93d8";
    default: return "#90a4ae";
  }
}

function App() {
  const [scanPath, setScanPath] = createSignal("C:\\");
  const [isScanning, setIsScanning] = createSignal(false);
  const [progress, setProgress] = createSignal<ScanProgress | null>(null);
  const [tree, setTree] = createSignal<FileNode | null>(null);
  const [viewMode, setViewMode] = createSignal<ViewMode>("tree");
  const [selectedNode, setSelectedNode] = createSignal<FileNode | null>(null);
  const [breadcrumbs, setBreadcrumbs] = createSignal<FileNode[]>([]);
  const [sunburstCenter, setSunburstCenter] = createSignal<FileNode | null>(null);
  const [sunburstRadius, setSunburstRadius] = createSignal(250);
  const [cleanupCandidates, setCleanupCandidates] = createSignal<CleanupCandidate[]>([]);
  const [showCleanup, setShowCleanup] = createSignal(false);
  const [cleanupMode, setCleanupMode] = createSignal<"preview" | "confirm" | "complete">("preview");
  const [selectedForCleanup, setSelectedForCleanup] = createSignal<Set<string>>(new Set());
  const [cleanupResult, setCleanupResult] = createSignal<{ removed: number; freed: number } | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  const startScan = async () => {
    try {
      setIsScanning(true);
      setTree(null);
      setSelectedNode(null);
      setBreadcrumbs([]);
      setError(null);

      await invoke("start_scan", {
        options: {
          root: scanPath(),
          follow_symlinks: false,
          max_depth: null,
          exclude_patterns: [],
        },
      });

      const poll = setInterval(async () => {
        try {
          const status = await invoke("scan_status");
          if (status.result) {
            setTree(status.result.root);
            setIsScanning(false);
            clearInterval(poll);
          } else if (status.error) {
            setError(status.error);
            setIsScanning(false);
            clearInterval(poll);
          } else {
            // Handle partial tree for incremental updates
            if (status.partial_tree) {
              setTree(status.partial_tree.root);
            }
            setProgress({
              path: status.current_path,
              files_scanned: status.files_scanned,
              bytes_scanned: status.bytes_scanned,
            });
          }
        } catch (e) {
          clearInterval(poll);
          setIsScanning(false);
        }
      }, 100);
    } catch (e) {
      setError(String(e));
      setIsScanning(false);
    }
  };

  const navigateTo = (node: FileNode) => {
    setSelectedNode(node);
    setBreadcrumbs([...breadcrumbs(), node]);
  };

  // Squarified treemap algorithm (Bruls, Huizing, van Wijk)
  const squarify = (children: FileNode[], w: number, h: number, x: number, y: number) => {
    if (!children.length || w <= 0 || h <= 0) return [];

    const total = children.reduce((sum, c) => sum + c.size, 0);
    if (total === 0) return [];

    // Sort children by size descending
    const sorted = [...children].sort((a, b) => b.size - a.size);
    
    const rects: { x: number; y: number; w: number; h: number; node: FileNode }[] = [];
    
    const layoutRow = (row: FileNode[], rx: number, ry: number, rw: number, rh: number, horizontal: boolean) => {
      const rowTotal = row.reduce((sum, c) => sum + c.size, 0);
      const rowSize = (rowTotal / total) * (horizontal ? rw : rh);
      
      let pos = 0;
      for (const child of row) {
        const childSize = (child.size / rowTotal) * rowSize;
        if (horizontal) {
          rects.push({ x: rx + pos, y: ry, w: childSize, h: rh, node: child });
          pos += childSize;
        } else {
          rects.push({ x: rx, y: ry + pos, w: rw, h: childSize, node: child });
          pos += childSize;
        }
      }
    };

    let remaining = [...sorted];
    let currentRow: FileNode[] = [];
    let currentRowSum = 0;
    let horizontal = w >= h;
    let cx = x, cy = y;
    let cw = w, ch = h;

    // Helper to compute worst aspect ratio in a row
    const worstAspectRatio = (row: FileNode[], rowSum: number, horizontal: boolean) => {
      if (!row.length) return Infinity;
      const rowTotal = rowSum;
      const rowSize = (rowTotal / total) * (horizontal ? cw : ch);
      const avgSize = rowSize / row.length;
      // Aspect ratio of each cell in the row
      const cellW = horizontal ? avgSize : rowSize;
      const cellH = horizontal ? rowSize : avgSize;
      return Math.max(cellW / cellH, cellH / cellW);
    };

    while (remaining.length > 0) {
      const child = remaining[0];
      const newRowSum = currentRowSum + child.size;
      
      // Check if adding this child improves the layout
      const currentWorst = worstAspectRatio(currentRow, currentRowSum, horizontal);
      const newWorst = worstAspectRatio([...currentRow, child], newRowSum, horizontal);

      if (currentRow.length > 0 && newWorst > currentWorst) {
        // Layout current row
        layoutRow(currentRow, cx, cy, cw, ch, horizontal);
        
        // Move to next row
        if (horizontal) {
          cy += (currentRowSum / total) * ch;
          ch = h - (cy - y);
        } else {
          cx += (currentRowSum / total) * cw;
          cw = w - (cx - x);
        }
        
        currentRow = [];
        currentRowSum = 0;
        horizontal = !horizontal;
      } else {
        currentRow.push(child);
        currentRowSum = newRowSum;
        remaining.shift();
      }
    }

    // Layout final row
    if (currentRow.length > 0) {
      layoutRow(currentRow, cx, cy, cw, ch, horizontal);
    }

    return rects;
  };

  const renderTreemap = (node: FileNode, x: number, y: number, w: number, h: number) => {
    if (!node.children.length || w < 2 || h < 2) return null;

    const rects = squarify(node.children, w, h, x, y);
    
    return rects.map(({ x, y, w, h, node: child }) => (
      <rect
        key={child.path}
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
  };

  const renderSunburst = (node: FileNode, cx: number, cy: number, r: number, startAngle: number, endAngle: number, isRoot: boolean = true) => {
    if (!node.children.length || r < 2) return null;

    const total = node.children.reduce((sum, c) => sum + c.size, 0);
    if (total === 0) return null;

    const paths: JSX.Element[] = [];
    let angle = startAngle;

    // Center circle for drill-up (if not root)
    if (!isRoot) {
      paths.push(
        <circle
          cx={cx}
          cy={cy}
          r={r * 0.6}
          fill="#16213e"
          stroke="#4fc3f7"
          stroke-width={2}
          opacity={0.9}
          style={{ cursor: "pointer" }}
          onClick={() => {
            if (breadcrumbs().length > 1) {
              const newBreadcrumbs = breadcrumbs().slice(0, -1);
              setBreadcrumbs(newBreadcrumbs);
              setSelectedNode(newBreadcrumbs[newBreadcrumbs.length - 1]);
            } else {
              setBreadcrumbs([]);
              setSelectedNode(tree());
            }
          }}
        />
      );
    }

    for (const child of node.children) {
      const ratio = child.size / total;
      const childAngle = (endAngle - startAngle) * ratio;

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

        paths.push(
          <path
            d={d}
            fill={getFileTypeColor(child.node_type)}
            stroke="#1a1a2e"
            stroke-width={1}
            opacity={0.8}
            style={{ cursor: "pointer" }}
            onClick={() => {
              if (child.node_type === "Directory" && child.children.length > 0) {
                // Drill down into directory
                setSunburstCenter(child);
                setSunburstRadius(r * 0.8);
              } else {
                navigateTo(child);
              }
            }}
          />
        );
      }

      angle += childAngle;
    }

    return paths;
  };

  const renderTreeNode = (node: FileNode, depth: number = 0) => {
    const percentage = tree() ? (node.size / tree()!.size * 100).toFixed(1) : "0";

    return (
      <div style={{ "padding-left": `${depth * 20}px` }}>
        <div
          class="tree-row"
          style={{ cursor: "pointer" }}
          onClick={() => navigateTo(node)}
        >
          <span style={{ color: getFileTypeColor(node.node_type), "margin-right": "8px" }}>
            {node.node_type === "Directory" ? "📁" : "📄"}
          </span>
          <span class="name">{node.name}</span>
          <span class="size">{formatSize(node.size)}</span>
          <span class="percentage">{percentage}%</span>
          <span class="count">{node.item_count} items</span>
        </div>
        <Show when={node.node_type === "Directory" && node.children.length > 0}>
          <For each={node.children.slice(0, 50)}>
            {(child) => renderTreeNode(child, depth + 1)}
          </For>
        </Show>
      </div>
    );
  };

  const loadCleanupCandidates = async () => {
    try {
      const candidates = await invoke<CleanupCandidate[]>("detect_cleanup_candidates", {
        path: scanPath(),
      });
      setCleanupCandidates(candidates);
      setShowCleanup(true);
    } catch (e) {
      setError(String(e));
    }
  };

  const executeCleanup = async () => {
    try {
      const paths = cleanupCandidates()
        .filter(c => selectedForCleanup().has(c.path))
        .map(c => c.path);
      
      const result = await invoke<{ removed: number; freed: number }>("cleanup_selected", { paths });
      setCleanupResult(result);
      setCleanupMode("complete");
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div class="app">
      <header>
        <h1>tpt-diskview</h1>
        <div class="controls">
          <input
            type="text"
            value={scanPath()}
            onInput={(e) => setScanPath(e.currentTarget.value)}
            placeholder="Path to scan..."
          />
          <button onClick={startScan} disabled={isScanning()}>
            {isScanning() ? "Scanning..." : "Start Scan"}
          </button>
          <button onClick={loadCleanupCandidates}>
            Detect Cleanup Candidates
          </button>
        </div>
      </header>

      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>

      <Show when={isScanning() && progress()}>
        <div class="progress">
          <div class="progress-bar">
            <div class="progress-fill" />
          </div>
          <div class="progress-text">
            Scanning: {progress()!.path} | {progress()!.files_scanned} files | {formatSize(progress()!.bytes_scanned)}
          </div>
        </div>
      </Show>

      <div class="main-content">
        <aside class="sidebar">
          <div class="view-tabs">
            <button
              class={viewMode() === "tree" ? "active" : ""}
              onClick={() => setViewMode("tree")}
            >
              Tree
            </button>
            <button
              class={viewMode() === "treemap" ? "active" : ""}
              onClick={() => setViewMode("treemap")}
            >
              Treemap
            </button>
            <button
              class={viewMode() === "sunburst" ? "active" : ""}
              onClick={() => setViewMode("sunburst")}
            >
              Sunburst
            </button>
          </div>

          <Show when={breadcrumbs().length > 0}>
            <div class="breadcrumbs">
              <For each={breadcrumbs()}>
                {(crumb, i) => (
                  <span
                    class="breadcrumb"
                    onClick={() => {
                      setBreadcrumbs(breadcrumbs().slice(0, i() + 1));
                      setSelectedNode(crumb);
                    }}
                  >
                    {crumb.name} /
                  </span>
                )}
              </For>
            </div>
          </Show>

          <Show when={tree()}>
            <div class="tree-view">
              {renderTreeNode(selectedNode() || tree()!)}
            </div>
          </Show>
        </aside>

        <main class="visualization">
          <Show when={viewMode() === "treemap" && tree()}>
            <svg width="100%" height="100%" viewBox="0 0 800 600">
              {renderTreemap(selectedNode() || tree()!, 0, 0, 800, 600)}
            </svg>
          </Show>

          <Show when={viewMode() === "sunburst" && tree()}>
            <svg width="100%" height="100%" viewBox="0 0 800 600">
              {renderSunburst(selectedNode() || tree()!, 400, 300, 250, 0, 2 * Math.PI)}
            </svg>
          </Show>

          <Show when={viewMode() === "tree" && tree()}>
            <div class="table-header">
              <span class="name">Name</span>
              <span class="size">Size</span>
              <span class="percentage">%</span>
              <span class="count">Items</span>
            </div>
            <div class="table-view">
              {renderTreeNode(selectedNode() || tree()!)}
            </div>
          </Show>
        </main>
      </div>

      <Show when={showCleanup()}>
        <div class="cleanup-panel">
          <div class="cleanup-header">
            <h2>Cleanup Candidates</h2>
            <button onClick={() => setShowCleanup(false)}>×</button>
          </div>
          
          <Show when={cleanupMode() === "preview"}>
            <div class="cleanup-mode-bar">
              <span>Preview Mode - Select items to clean up</span>
              <button class="primary" onClick={() => setCleanupMode("confirm")}>
                Review Selection ({selectedForCleanup().size} items)
              </button>
            </div>
            <div class="cleanup-list">
              <For each={cleanupCandidates()}>
                {(candidate) => {
                  const isSelected = selectedForCleanup().has(candidate.path);
                  return (
                    <div class="cleanup-item" style={{ opacity: isSelected ? 1 : 0.6 }}>
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => {
                          const newSet = new Set(selectedForCleanup());
                          if (isSelected) {
                            newSet.delete(candidate.path);
                          } else {
                            newSet.add(candidate.path);
                          }
                          setSelectedForCleanup(newSet);
                        }}
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
              Selected: {formatSize(
                cleanupCandidates()
                  .filter(c => selectedForCleanup().has(c.path))
                  .reduce((sum, c) => sum + c.size, 0)
              )}
            </div>
          </Show>

          <Show when={cleanupMode() === "confirm"}>
            <div class="cleanup-mode-bar warning">
              <span>⚠ Confirm Deletion - This will move {selectedForCleanup().size} items to Recycle Bin</span>
            </div>
            <div class="cleanup-list">
              <For each={cleanupCandidates().filter(c => selectedForCleanup().has(c.path))}>
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
                Move to Recycle Bin ({formatSize(
                  cleanupCandidates()
                    .filter(c => selectedForCleanup().has(c.path))
                    .reduce((sum, c) => sum + c.size, 0)
                )})
              </button>
            </div>
          </Show>

          <Show when={cleanupMode() === "complete" && cleanupResult()}>
            <div class="cleanup-mode-bar success">
              <span>✓ Cleanup Complete - Removed {cleanupResult()!.removed} items, freed {formatSize(cleanupResult()!.freed)}</span>
            </div>
            <div class="cleanup-actions">
              <button onClick={() => {
                setCleanupMode("preview");
                setSelectedForCleanup(new Set());
                setCleanupResult(null);
                loadCleanupCandidates();
              }}>
                Done
              </button>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={selectedNode()}>
        <div class="details-panel">
          <h3>{selectedNode()!.name}</h3>
          <p>Path: {selectedNode()!.path}</p>
          <p>Size: {formatSize(selectedNode()!.size)}</p>
          <p>Items: {selectedNode()!.item_count}</p>
          <p>Type: {selectedNode()!.node_type}</p>
        </div>
      </Show>
    </div>
  );
}

export default App;
