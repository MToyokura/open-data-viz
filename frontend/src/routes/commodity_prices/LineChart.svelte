<script lang="ts">
  import { browser } from "$app/environment";
  import Chart, { type ChartData } from "chart.js/auto";

  let props: {
    chartData: ChartData;
  } = $props();

  let canvas: HTMLCanvasElement;
  let myChart: Chart | null = null;

  $effect(() => {
    // Always destroy the existing chart instance before potentially creating a new one
    if (myChart) {
      myChart.destroy();
      myChart = null;
    }

    if (!browser || !props.chartData) {
      return;
    }

    // Dynamically import because `ReferenceError: window is not defined`
    import("chartjs-plugin-zoom")
      .then((module) => {
        const zoomPlugin = module.default;
        Chart.register(zoomPlugin);

        const ctx = canvas.getContext("2d");
        if (!ctx) {
          console.error("Failed to get canvas context");
          return;
        }

        // Create the new chart instance with the updated data
        myChart = new Chart(ctx, {
          type: "line",
          data: props.chartData,
          options: {
            animation: false,
            elements: {
              line: {
                tension: 0,
              },
            },
            interaction: {
              mode: "nearest",
              axis: "x",
              intersect: false,
            },
            plugins: {
              zoom: {
                zoom: {
                  wheel: {
                    enabled: true,
                  },
                  pinch: {
                    enabled: true,
                  },
                  mode: "x",
                },
                pan: {
                  enabled: true,
                  mode: "x",
                },
              },
            },
          },
        });
      })
      .catch((error) => {
        console.error("Failed to load chartjs-plugin-zoom:", error);
      });

    // Return a cleanup function that destroys the chart when the component is unmounted
    return () => {
      if (myChart) {
        myChart.destroy();
        myChart = null;
      }
    };
  });
</script>

<div id="chart-container">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  #chart-container {
    margin: 2rem;
    padding: 2rem;
    border: 2px solid #ccc;
    border-radius: 8px;
  }

  @media (max-width: 48rem) {
    #chart-container {
      margin: 2rem 0;
    }
  }

  canvas {
    width: 100%;
    max-height: 60vh;
  }
</style>
