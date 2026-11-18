import { invoke } from "@tauri-apps/api/core";

// Where the report is shown.
const out = document.querySelector<HTMLPreElement>("#out")!;

// Run the checks in Rust and show the report or the error.
document.querySelector("#run")!.addEventListener("click", async () => {
  out.textContent = "Running…";

  try {
    out.textContent = await invoke<string>("run_spike_checks");
  } catch (e) {
    out.textContent = `Error: ${e}`;
  }
});
