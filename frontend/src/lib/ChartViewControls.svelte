<script>
  /** @type {{ mode: "fit"|"scroll", onchange: (mode: "fit"|"scroll") => void }} */
  let { mode = $bindable(), onchange } = $props();

  /** @param {"fit"|"scroll"} nextMode */
  function selectMode(nextMode) {
    mode = nextMode;
    onchange(nextMode);
  }
</script>

<div class="chart-view-controls" role="group" aria-label="グラフの表示方法">
  <button
    type="button"
    class:active={mode === "fit"}
    aria-pressed={mode === "fit"}
    onclick={() => selectMode("fit")}>画面幅表示</button
  >
  <button
    type="button"
    class:active={mode === "scroll"}
    aria-pressed={mode === "scroll"}
    onclick={() => selectMode("scroll")}>スクロール表示</button
  >
</div>

<style>
  .chart-view-controls {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
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

  button.active {
    border-color: var(--focus-color);
    background: var(--surface-muted-color);
    font-weight: 600;
  }
</style>
