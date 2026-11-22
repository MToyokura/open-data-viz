<script>
  import { onMount } from "svelte";

  /** @type {{ chartData: import("echarts").EChartsOption, label: string }} */
  let { chartData, label } = $props();
  /** @type {HTMLDivElement} */
  let container;
  let chart = $state.raw(
    /** @type {import("echarts").EChartsType | null} */ (null),
  );
  let failed = $state(false);

  onMount(() => {
    let disposed = false;
    /** @type {ResizeObserver | undefined} */
    let observer;

    import("echarts")
      .then(({ init }) => {
        if (disposed) return;
        const instance = init(container);
        chart = instance;
        observer = new ResizeObserver(() => instance.resize());
        observer.observe(container);
      })
      .catch(() => {
        if (!disposed) failed = true;
      });

    return () => {
      disposed = true;
      observer?.disconnect();
      chart?.dispose();
    };
  });

  $effect(() => {
    chart?.setOption(chartData, { notMerge: true });
  });
</script>

<div class="chart-frame">
  <div
    class="chart-canvas"
    bind:this={container}
    role="img"
    aria-label={label}
  ></div>
  {#if failed}
    <div class="chart-state" role="alert">グラフを表示できませんでした。</div>
  {/if}
</div>
