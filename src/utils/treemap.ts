import type { FileNode } from "../types";

export interface TreemapRect {
  x: number;
  y: number;
  w: number;
  h: number;
  node: FileNode;
}

export function squarify(
  children: FileNode[],
  w: number,
  h: number,
  x: number,
  y: number
): TreemapRect[] {
  if (!children.length || w <= 0 || h <= 0) return [];

  const total = children.reduce((sum, c) => sum + c.size, 0);
  if (total === 0) return [];

  const sorted = [...children].sort((a, b) => b.size - a.size);

  const rects: TreemapRect[] = [];

  const layoutRow = (
    row: FileNode[],
    rx: number,
    ry: number,
    rw: number,
    rh: number,
    horizontal: boolean
  ) => {
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

  const worstAspectRatio = (row: FileNode[], rowSum: number, horizontal: boolean) => {
    if (!row.length) return Infinity;
    const rowTotal = rowSum;
    const rowSize = (rowTotal / total) * (horizontal ? cw : ch);
    const avgSize = rowSize / row.length;
    const cellW = horizontal ? avgSize : rowSize;
    const cellH = horizontal ? rowSize : avgSize;
    return Math.max(cellW / cellH, cellH / cellW);
  };

  while (remaining.length > 0) {
    const child = remaining[0];
    const newRowSum = currentRowSum + child.size;

    const currentWorst = worstAspectRatio(currentRow, currentRowSum, horizontal);
    const newWorst = worstAspectRatio([...currentRow, child], newRowSum, horizontal);

    if (currentRow.length > 0 && newWorst > currentWorst) {
      layoutRow(currentRow, cx, cy, cw, ch, horizontal);

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

  if (currentRow.length > 0) {
    layoutRow(currentRow, cx, cy, cw, ch, horizontal);
  }

  return rects;
}
