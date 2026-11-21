<script>
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import LicenseLink from "#lib/LicenseLink.svelte";
  import LoadState from "#lib/LoadState.svelte";
  import { VITE_STORAGE_BASE_URL } from "#lib/paths.js";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import ReferenceTable from "#lib/ReferenceTable.svelte";
  import { useResponsivePanels } from "#lib/useResponsivePanels.svelte.js";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import { onMount } from "svelte";

  /** @typedef {{id:string,label:string,groupPath:string[],type:string,values:(number|null)[]}} FisherySeries */
  /** @typedef {{years:number[],series:FisherySeries[],notes:(string|{text:string})[]}} FisheryData */

  const sourceUrl =
    "https://www.e-stat.go.jp/stat-search/files?tclass=000001024930&cycle=0";
  let data = $state.raw(
    /** @type {FisheryData} */ ({ years: [], series: [], notes: [] }),
  );
  let status = $state(/** @type {"loading"|"loaded"|"error"} */ ("loading"));
  let error = $state("");
  let query = $state("");
  let selectedIds = $state(["total"]);
  const panels = useResponsivePanels(() => selectedIds.length, {
    mobileOpen: true,
  });
  let chartNode = $state(/** @type {HTMLDivElement | undefined} */ (undefined));
  let chart = $state.raw(
    /** @type {import("echarts").EChartsType | null} */ (null),
  );

  let series = $derived(data.series);
  let filteredSeries = $derived(
    series.filter((item) =>
      `${item.label} ${item.groupPath?.join(" ") ?? ""}`
        .toLocaleLowerCase("ja")
        .includes(query.toLocaleLowerCase("ja")),
    ),
  );
  let selected = $derived(
    selectedIds.flatMap((id) => series.filter((item) => item.id === id)),
  );
  /** @param {FisherySeries} item */
  function seriesLabel(item) {
    return item.type === "total" ? item.label : item.groupPath.join(" › ");
  }
  let chartOption = $derived({
    animation: false,
    tooltip: {
      trigger: "axis",
      valueFormatter: /** @param {number|null} value */ (value) =>
        value == null
          ? "データなし"
          : `${Number(value).toLocaleString("ja-JP")} t`,
    },
    legend: { type: "scroll", bottom: 0 },
    grid: { left: 76, right: 20, top: 24, bottom: 78 },
    xAxis: {
      type: "category",
      data: data.years,
      name: "年",
      boundaryGap: false,
    },
    yAxis: {
      type: "value",
      name: "漁獲量（t）",
      axisLabel: {
        formatter: /** @param {number} value */ (value) =>
          Number(value).toLocaleString("ja-JP"),
      },
    },
    series: selected.map((item) => ({
      name: seriesLabel(item),
      type: "line",
      showSymbol: false,
      connectNulls: false,
      data: item.values,
    })),
  });

  async function loadData() {
    status = "loading";
    error = "";
    try {
      const response = await fetch(
        `${VITE_STORAGE_BASE_URL}/marine_fishery_catch/transformed_data/catches.json`,
      );
      if (!response.ok)
        throw new Error(`データの取得に失敗しました（${response.status}）`);
      /** @type {FisheryData} */
      const payload = await response.json();
      if (!Array.isArray(payload.years) || !Array.isArray(payload.series))
        throw new Error("データ形式を確認できませんでした");
      data = payload;
      selectedIds = [
        payload.series.find((item) => item.id === "total")?.id ??
          payload.series[0]?.id,
      ].filter(Boolean);
      status = "loaded";
    } catch (cause) {
      error =
        cause instanceof Error ? cause.message : "データを取得できませんでした";
      status = "error";
    }
  }

  /** @param {string} id */
  function toggleSeries(id) {
    if (selectedIds.includes(id)) {
      selectedIds = selectedIds.filter((selectedId) => selectedId !== id);
    } else {
      const replacingInitialTotal =
        selected.length === 1 && selected[0].type === "total";
      selectedIds = replacingInitialTotal ? [id] : [...selectedIds, id];
    }
  }

  /** @param {string} id */
  function removeSeries(id) {
    selectedIds = selectedIds.filter((selectedId) => selectedId !== id);
  }

  /** @param {string | {text:string}} note */
  function noteText(note) {
    const text = typeof note === "string" ? note : note.text;
    return text.replace(/^\s*\d+[.．]\s*/, "");
  }

  onMount(() => {
    let disposed = false;
    /** @type {ResizeObserver | undefined} */
    let observer;
    loadData();
    import("echarts").then(({ init }) => {
      if (disposed || !chartNode) return;
      chart = init(chartNode);
      observer = new ResizeObserver(() => chart?.resize());
      observer.observe(chartNode);
    });
    return () => {
      disposed = true;
      observer?.disconnect();
      chart?.dispose();
      chart = null;
    };
  });

  $effect(() => {
    const option = chartOption;
    if (chart && status === "loaded")
      chart.setOption(option, { notMerge: true });
  });
</script>

<svelte:head>
  <title>海面漁業魚種別漁獲量 | Open Data Viz</title>
  <meta
    name="description"
    content="1956年から2024年までの全国の海面漁業漁獲量を魚種別に比較できます。"
  />
</svelte:head>

<div class="visualization-page visualization-page--chart-first">
  <main class="page-content">
    <WorkspaceHeader title="海面漁業魚種別漁獲量" />
    <div class="workspace">
      <aside class="details-column" aria-label="系列の選択とデータ情報">
        {#if status !== "loaded"}
          <LoadState
            state={status}
            message={status === "loading"
              ? "漁獲量データを読み込んでいます"
              : error}
            onretry={status === "error" ? loadData : undefined}
          />
        {:else}
          <section class="control-panel" aria-label="魚種の選択">
            <details class="selector-panel" bind:open={panels.isSelectorOpen}>
              <summary>魚種を選択</summary>
              <div class="selector-panel-content ui-panel">
                <label for="series-search">魚種・分類を検索</label>
                <input
                  id="series-search"
                  type="search"
                  bind:value={query}
                  placeholder="例：さんま"
                />
                <p class="hint">選択した魚種をグラフで比較できます。</p>
                <div class="options" role="group" aria-label="魚種・分類">
                  {#each filteredSeries as item (item.id)}
                    <label class="option">
                      <input
                        type="checkbox"
                        checked={selectedIds.includes(item.id)}
                        onchange={() => toggleSeries(item.id)}
                      />
                      <span
                        ><small>{item.groupPath?.join(" › ")}</small
                        >{item.label}</span
                      >
                    </label>
                  {:else}
                    <p class="hint">該当する魚種・分類がありません。</p>
                  {/each}
                </div>
              </div>
            </details>
          </section>
          <section class="comparison-panel" aria-label="選択中の系列">
            <details class="added-items" bind:open={panels.isComparisonOpen}>
              <summary>
                <h2>選択中の系列 <span>{selected.length}系列</span></h2>
              </summary>
              <fieldset class="selected-list">
                <legend>選択中の系列</legend>
                {#if selected.length > 0}
                  <ul>
                    {#each selected as item (item.id)}
                      <li>
                        <span>{seriesLabel(item)}</span>
                        <button
                          type="button"
                          class="icon-button"
                          aria-label="{item.label}を選択中一覧から削除"
                          onclick={() => removeSeries(item.id)}
                          ><img
                            src={deleteIcon}
                            alt=""
                            aria-hidden="true"
                          /></button
                        >
                      </li>
                    {/each}
                  </ul>
                {:else}
                  <p class="empty-comparison">魚種を選択してください。</p>
                {/if}
              </fieldset>
              <button
                type="button"
                class="reset-button"
                disabled={selectedIds.length === 0}
                onclick={() => (selectedIds = [])}>すべて消去</button
              >
            </details>
          </section>
          <ReferencePanel title="データについて">
            <h3>表示について</h3>
            <ul class="reference-notes">
              <li>
                出典に掲載された合計・小計・魚種別の値をそのまま表示します。
              </li>
              <li>
                「…」はデータなしです。2011年の震災後、岩手・宮城・福島県の調査データが欠ける年があります。
              </li>
              {#if selected.some((item) => item.id.includes("kuro") || item.label.includes("くろまぐろ"))}
                <li>くろまぐろは1994年までみなみまぐろを含む定義です。</li>
              {/if}
            </ul>
            <details class="source-notes">
              <summary
                >原資料の注記をすべて表示（{data.notes.length}件）</summary
              >
              <ol>
                {#each data.notes as note, index (index)}
                  <li>{noteText(note)}</li>
                {/each}
              </ol>
            </details>
            <h3>出典</h3>
            <ReferenceTable
              rows={[
                {
                  label: "統計名",
                  value:
                    "漁業・養殖業生産統計年報「海面漁業魚種別漁獲量累年統計（全国）」",
                },
                {
                  label: "対象期間",
                  value: `${data.years[0]}〜${data.years.at(-1)}年（単位：トン）`,
                },
                {
                  label: "公開表",
                  url: sourceUrl,
                  linkLabel: "e-Statで統計表を見る",
                },
              ]}
            />
          </ReferencePanel>
          <LicenseLink />
        {/if}
      </aside>
      <section
        class="chart-panel"
        aria-label="漁獲量グラフ"
        aria-busy={status !== "loaded"}
      >
        <div
          bind:this={chartNode}
          class="chart"
          role="img"
          aria-label="選択した魚種の年別漁獲量を示す折れ線グラフ"
        ></div>
        {#if status === "loaded"}
          <div class="data-table-wrapper">
            <table>
              <caption>年別漁獲量（トン）</caption>
              <thead>
                <tr>
                  <th scope="col">年</th>
                  {#each selected as item (item.id)}
                    <th scope="col">{seriesLabel(item)}</th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each data.years as year, index (year)}
                  <tr>
                    <th scope="row">{year}</th>
                    {#each selected as item (item.id)}
                      <td
                        >{item.values[index] == null
                          ? "データなし"
                          : Number(item.values[index]).toLocaleString(
                              "ja-JP",
                            )}</td
                      >
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .chart-panel {
    min-width: 0;
  }
  .chart-panel {
    --chart-height: calc(100dvh - 12rem);
  }
  .selector-panel-content > label {
    display: block;
    font-weight: 600;
    margin-bottom: var(--space-2);
  }
  input[type="search"] {
    box-sizing: border-box;
    width: 100%;
    min-height: var(--control-height);
    padding: var(--space-2);
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-color);
    color: inherit;
  }
  .hint,
  .option small {
    color: var(--text-muted-color);
  }
  .options {
    max-height: min(28rem, 55vh);
    overflow: auto;
    margin-top: var(--space-3);
    border-top: var(--border-default);
  }
  .option {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-2) 0;
    cursor: pointer;
  }
  .option + .option {
    border-top: 1px solid var(--border-color);
  }
  .option span {
    display: grid;
    gap: 0.15rem;
  }
  .option small {
    line-height: 1.35;
  }
  .chart {
    height: var(--chart-height);
    min-height: 30rem;
    width: 100%;
  }
  .data-table-wrapper {
    max-width: 100%;
    overflow-x: auto;
    margin-top: var(--space-3);
  }
  table {
    border-collapse: collapse;
    min-width: 100%;
    text-align: right;
  }
  caption {
    text-align: left;
    font-weight: 600;
    margin-bottom: var(--space-2);
  }
  th,
  td {
    padding: var(--space-2);
    border-bottom: var(--border-default);
    white-space: nowrap;
  }
  thead th:first-child,
  tbody th {
    text-align: left;
  }
  .source-notes {
    margin-top: var(--space-3);
    border-top: var(--border-default);
    padding-top: var(--space-3);
  }
  .source-notes summary {
    cursor: pointer;
    font-weight: 600;
  }
  .source-notes ol {
    max-height: 28rem;
    overflow: auto;
    padding-left: 1.5rem;
  }
  .source-notes li + li {
    margin-top: var(--space-2);
  }
  @media (max-width: 50rem) {
    .options {
      max-height: 16rem;
    }
    .chart-panel {
      --chart-height: 55vh;
    }
    .chart {
      min-height: 20rem;
    }
  }
</style>
