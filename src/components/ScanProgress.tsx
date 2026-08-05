import { Show } from "solid-js";
import { state } from "../stores/scanStore";
import { formatSize } from "../utils/format";

export default function ScanProgress() {
  return (
    <Show when={state.isScanning && state.progress}>
      <div class="progress">
        <div class="progress-bar">
          <div class="progress-fill" />
        </div>
        <div class="progress-text">
          Scanning: {state.progress!.path} | {state.progress!.files_scanned} files | {formatSize(state.progress!.bytes_scanned)}
        </div>
      </div>
    </Show>
  );
}
