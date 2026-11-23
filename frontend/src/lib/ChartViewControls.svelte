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
