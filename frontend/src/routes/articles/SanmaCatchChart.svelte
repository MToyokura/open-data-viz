<script>
  import { onMount } from "svelte";

  /** @typedef {import("echarts").EChartsType} EChartsType */

  const catchData = [
    [2000, 216471],
    [2001, 269797],
    [2002, 205282],
    [2003, 264804],
    [2004, 204371],
    [2005, 234451],
    [2006, 244586],
    [2007, 296521],
    [2008, 354727],
    [2009, 310744],
    [2010, 207488],
    [2011, 215353],
    [2012, 221470],
    [2013, 149853],
    [2014, 228647],
    [2015, 116243],
    [2016, 113828],
    [2017, 83803],
    [2018, 128929],
    [2019, 45778],
    [2020, 29675],
    [2021, 19513],
    [2022, 18384],
    [2023, 25753],
    [2024, 39335],
  ];

  let container;

  onMount(() => {
    let disposed = false;
    /** @type {EChartsType | null} */
    let chart = null;
    /** @type {ResizeObserver | undefined} */
    let resizeObserver;

    import("echarts")
      .then(({ init }) => {
        if (disposed) return;

        chart = init(container);
        chart.setOption({
          animation: false,
          aria: { enabled: true },
          tooltip: { trigger: "axis" },
          grid: { left: 76, right: 24, top: 24, bottom: 48 },
          xAxis: {
            type: "category",
            data: catchData.map(([year]) => String(year)),
            axisLabel: {
              interval: 0,
              formatter: (year, index) =>
                index % 5 === 0 || index === catchData.length - 1 ? year : "",
            },
          },
          yAxis: {
            type: "value",
            name: "漁獲量（t）",
            nameLocation: "middle",
            nameGap: 56,
            axisLabel: {
              formatter: (value) => Number(value).toLocaleString("ja-JP"),
            },
          },
          series: [
            {
              type: "bar",
              name: "さんま漁獲量",
              data: catchData.map(([, value]) => value),
              itemStyle: { color: "#5275e8" },
              barMaxWidth: 18,
            },
          ],
        });

        resizeObserver = new ResizeObserver(() => chart?.resize());
        resizeObserver.observe(container);
      })
      .catch((error) => {
        console.error("Failed to load Apache ECharts:", error);
      });

    return () => {
      disposed = true;
      resizeObserver?.disconnect();
      chart?.dispose();
      chart = null;
    };
  });
</script>

<figure class="article-chart" aria-label="さんまの漁獲量の推移">
  <div class="chart-frame">
    <div
      bind:this={container}
      class="chart-canvas sanma-catch-chart"
      role="img"
      aria-label="さんま漁獲量を示す棒グラフ"
    ></div>
  </div>
  <figcaption>さんまの漁獲量の推移</figcaption>
</figure>
