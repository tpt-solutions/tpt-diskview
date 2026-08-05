import { Show } from "solid-js";
import { state, setError } from "../stores/scanStore";

export default function ErrorBar() {
  return (
    <Show when={state.error}>
      <div class="error">
        {state.error}
        <button
          onClick={() => setError(null)}
          style={{
            float: "right",
            background: "none",
            border: "none",
            color: "#ff5252",
            padding: "0 4px",
            cursor: "pointer",
          }}
        >
          ×
        </button>
      </div>
    </Show>
  );
}
