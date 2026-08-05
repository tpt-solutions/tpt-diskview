export function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

export function getFileTypeColor(type: string): string {
  switch (type) {
    case "Directory": return "#4fc3f7";
    case "File": return "#81c784";
    case "Symlink": return "#ffb74d";
    case "Junction": return "#ce93d8";
    default: return "#90a4ae";
  }
}
