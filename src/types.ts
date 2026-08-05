export interface FileNode {
  name: string;
  path: string;
  size: number;
  allocated: number;
  node_type: "Directory" | "File" | "Symlink" | "Junction";
  children: FileNode[];
  item_count: number;
  last_modified: number | null;
}

export interface ScanProgress {
  path: string;
  files_scanned: number;
  bytes_scanned: number;
}

export interface PartialTree {
  root: FileNode;
}

export interface CleanupCandidate {
  path: string;
  size: number;
  category: "TempFiles" | "Duplicates" | "DockerVolumes" | "BrowserCache";
  description: string;
}

export type ViewMode = "treemap" | "sunburst" | "tree";
