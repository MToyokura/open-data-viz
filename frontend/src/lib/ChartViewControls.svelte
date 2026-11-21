<script>
  /** @type {{ mode: "fit"|"scroll", onchange: (mode: "fit"|"scroll") => void }} */
  let { mode = $bindable(), onchange } = $props();

  function toggleMode() {
    mode = mode === "fit" ? "scroll" : "fit";
    onchange(mode);
  }
</script>

<div class="chart-view-controls">
  <button type="button" aria-pressed={mode === "fit"} onclick={toggleMode}
    >画面幅に収める</button
  >
</div>

<style>
  .chart-view-controls {
    display: flex;
    justify-content: flex-end;
    margin-top: var(--space-4);
  }

  button {
    padding: var(--space-1) var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-color);
    font-size: 0.875rem;
    cursor: pointer;
  }

  button[aria-pressed="true"] {
    border-color: var(--focus-color);
    background: var(--surface-muted-color);
    font-weight: 600;
  }
</style>
