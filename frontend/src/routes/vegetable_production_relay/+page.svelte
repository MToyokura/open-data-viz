<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import ReferenceTable from "#lib/ReferenceTable.svelte";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import LoadState from "#lib/LoadState.svelte";
  /** @typedef {import("./constants/types.js").PrefectureMonth} PrefectureMonth */
  import RelayMap from "./RelayMap.svelte";
  import VegetableSankey from "./VegetableSankey.svelte";
  import { aggregateSankey } from "./utils/aggregateSankey.js";
  import { useRelayData } from "./utils/useRelayData.svelte.js";

  const MIN_MARKER_SCALE = 1;
  const DEFAULT_MARKER_SCALE = 5;

  const relayData = useRelayData();
  let aggregation = $derived(relayData.aggregation);
  let dataError = $derived(relayData.error?.message ?? "");
  let years = $derived(aggregation?.years ?? []);
  let selectedProductLabel = $derived(
    relayData.products.find(
      (product) => product.key === relayData.selectedProduct,
    )?.label ?? "野菜",
  );
  let geojson = $derived(relayData.geojson);
  let mapError = $derived(relayData.mapError?.message ?? "");
  let months = $derived(
    [
      ...new Set(
        years.flatMap((item) => item.months.map((month) => month.month)),
      ),
    ].sort(),
  );
  let selectedMonth = $state("2021-01");
  let selectedPrefecture = $state("");
  let scale = $state(DEFAULT_MARKER_SCALE);
  let markerShape = $state("circle");

  function reloadPage() {
    window.location.reload();
  }

  let yearData = $derived(
    years.find((item) => item.year === Number(selectedMonth.slice(0, 4))),
  );
  let referenceRows = $derived([
    { label: "統計名", value: "青果物卸売市場調査" },
    { label: "表題", value: selectedProductLabel },
    {
      label: "統計表ID",
      ...(yearData
        ? {
            id: yearData.table_id,
            url: `https://www.e-stat.go.jp/dbview?sid=${yearData.table_id}`,
            linkLabel: "e-Statで統計表を見る",
          }
        : {
            value: relayData.isLoading
              ? "取得中…"
              : "選択年の統計表情報を取得できませんでした",
          }),
    },
    {
      label: "対象年／期間",
      value: yearData
        ? `${yearData.year}年（月次）`
        : relayData.isLoading
          ? "取得中…"
          : "選択年の期間情報を取得できませんでした",
    },
    {
      label: "表分類",
      value: "野菜の主要消費地域別・産地別の卸売数量及び卸売価格",
    },
    {
      label: "地図データ",
      value: "国土交通省「国土数値情報 行政区域データ（N03）」2026年版",
      links: [
        {
          label: "CC BY 4.0",
          href: "https://creativecommons.org/licenses/by/4.0/deed.ja",
        },
        {
          label: "国土数値情報 行政区域データ（2026年版）",
          href: "https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2026.html",
        },
      ],
    },
  ]);
  let monthIndex = $derived(months.indexOf(selectedMonth));
  let monthObservations = $derived(
    yearData?.months.filter((item) => item.month === selectedMonth) ?? [],
  );
  let observationsByPrefecture = $derived(
    Object.fromEntries(
      monthObservations.map((item) => [item.consignor_prefecture_name, item]),
    ),
  );
  let sankeyData = $derived(
    relayData.sankeyDetails.length === 47
      ? aggregateSankey(
          relayData.sankeyDetails,
          selectedMonth,
          monthObservations,
        )
      : undefined,
  );
  let selectedObservation = $derived(
    observationsByPrefecture[selectedPrefecture],
  );
  let excludedCategoryCounts = $derived(
    selectedObservation
      ? [
          {
            label: "表示単位未満",
            count: selectedObservation.below_display_unit_cells,
          },
          { label: "該当なし", count: selectedObservation.no_fact_cells },
          {
            label: "不詳・調査なし",
            count: selectedObservation.unknown_or_uncollected_cells,
          },
        ].filter((category) => category.count > 0)
      : [],
  );
  let detailContributions = $derived.by(() => {
    const currentDetail = relayData.detail;
    if (
      !currentDetail ||
      currentDetail.commodity !== relayData.selectedProduct ||
      currentDetail.consignor_prefecture_code !==
        selectedObservation?.consignor_prefecture_code
    )
      return undefined;
    return currentDetail.months.find((row) => row.month === selectedMonth)
      ?.contributions;
  });

  /** @param {number} index */
  function changeMonth(index) {
    if (index >= 0 && index < months.length) selectedMonth = months[index];
  }
  /** @param {KeyboardEvent} event */
  function handleMonthKeydown(event) {
    if (
      event.defaultPrevented ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey ||
      event.shiftKey
    )
      return;

    const target = event.target;
    if (
      target instanceof Element &&
      target.closest(
        "input, select, textarea, button, a, [contenteditable='true'], [role='button'], [tabindex]",
      )
    )
      return;

    if (event.key === "ArrowLeft" && monthIndex > 0) {
      event.preventDefault();
      changeMonth(monthIndex - 1);
    } else if (event.key === "ArrowRight" && monthIndex < months.length - 1) {
      event.preventDefault();
      changeMonth(monthIndex + 1);
    }
  }
  /** @param {string} name */
  function loadPrefectureDetail(name) {
    const code = observationsByPrefecture[name]?.consignor_prefecture_code;
    if (code) void relayData.selectPrefectureDetail(code);
  }

  /** @param {string} name */
  function selectPrefecture(name) {
    selectedPrefecture = name;
    loadPrefectureDetail(name);
  }

  function retryDetail() {
    loadPrefectureDetail(selectedPrefecture);
  }

  /** @param {number} value */
  function normalizeScale(value) {
    return Math.max(MIN_MARKER_SCALE, Number(value) || DEFAULT_MARKER_SCALE);
  }

  /** @param {number} amount */
  function adjustScale(amount) {
    scale = normalizeScale(Number((scale + amount).toFixed(1)));
  }
  const quantityFormatter = new Intl.NumberFormat("ja-JP", {
    maximumFractionDigits: 2,
  });
  /** @param {string} value */
  function formatValue(value) {
    return quantityFormatter.format(Number(value));
  }
  /** @param {PrefectureMonth | undefined} row */
  function valueLabel(row) {
    if (!row) return "値なし";
    if (row.reported_cells === 0) {
      return row.below_display_unit_cells > 0
        ? `数値の報告なし（表示単位未満 ${row.below_display_unit_cells} 区分）`
        : "数値の報告なし";
    }
    return `${formatValue(row.reported_value_sum)} ${row.source_unit}`;
  }
  /** @param {import("./constants/types.js").Contribution} cell */
  function contributionDisplay(cell) {
    if (cell.value_kind === "reported") {
      return { value: formatValue(cell.raw_value), note: "" };
    }
    if (cell.value_kind === "below_display_unit") {
      return { value: "0", note: "表示単位未満" };
    }
    if (cell.value_kind === "unknown_or_uncollected") {
      return { value: "…", note: "不詳・調査なし" };
    }
    return { value: "—", note: "該当なし" };
  }
  /** @param {string} month */
  const displayMonth = (month) => {
    const [year, number] = month.split("-");
    return `${year}年 ${Number(number)}月`;
  };
</script>

<svelte:window onkeydown={handleMonthKeydown} />

<svelte:head>
  <title>野菜の産地リレー | Open Data Viz</title>
</svelte:head>

{#snippet relayReference()}
  <ReferencePanel title="データについて">
    <h3>利用上の注意</h3>
    <ul class="reference-notes">
      <li>
        報告値合計は、産地都道府県ごとに、主要消費地域9区分のうち数値が報告された区分だけを合計したものです。
        表示単位未満、該当なし、不詳・調査なしの区分は含めていません。農業生産量や全国計を示す値ではありません。
      </li>
      <li>
        令和4年（2022年）に年間取扱量等調査の対象が中央卸売市場の卸売会社のみに見直されました。年をまたぐ比較にはご注意ください。
      </li>
    </ul>
    <h3>出典</h3>
    <ReferenceTable rows={referenceRows} />
  </ReferencePanel>
  <LicenseLink />
{/snippet}

<div class="page-shell visualization-page visualization-page--relay">
  <main class="page-content">
    <WorkspaceHeader title="野菜の産地リレー" />

    {#if aggregation || relayData.isLoading}
      <div class="workspace">
        <section class="map-column" aria-label="出荷数量マップ">
          {#if dataError}
            <LoadState
              state="error"
              message={dataError}
              onretry={reloadPage}
              retryLabel="ページを再読み込み"
            />
          {/if}
          <div class="map-heading">
            <div>
              <h2>
                {selectedProductLabel} · {displayMonth(selectedMonth)}の出荷数量
              </h2>
              <p>マーカーの面積は報告値の合計に比例します</p>
              <p class="map-instructions">
                都道府県またはマーカーを選ぶと、詳細パネルに報告値と消費地域別の内訳を表示します。地図上のスクロールで拡大・縮小できます。
              </p>
            </div>
          </div>
          <div class="map-view" aria-busy={relayData.isLoading}>
            {#if relayData.isLoading && geojson && relayData.selectedProduct}
              <div class="map-loading">
                <LoadState
                  state="loading"
                  message="出荷データを読み込んでいます…"
                />
              </div>
            {/if}
            {#if mapError}
              <LoadState
                state="error"
                message={mapError}
                onretry={reloadPage}
                retryLabel="ページを再読み込み"
              />
            {:else if !geojson || !relayData.selectedProduct}
              <LoadState state="loading" message="地図を読み込んでいます…" />
            {:else}
              <RelayMap
                {geojson}
                selectedProduct={relayData.selectedProduct}
                {selectedMonth}
                {observationsByPrefecture}
                sourceUnit={yearData?.source_unit ?? null}
                {scale}
                {markerShape}
                onSelect={selectPrefecture}
                --relay-map-height="100%"
                --relay-map-min-height="0"
                --relay-map-radius="0"
              />
            {/if}
          </div>
        </section>
        <section class="details-column" aria-label="詳細情報">
          <fieldset class="controls ui-panel">
            <legend>表示設定</legend>
            <label class="vegetable-control">
              野菜
              <select
                value={relayData.selectedProduct}
                onchange={(event) =>
                  relayData.selectProduct(event.currentTarget.value)}
                disabled={relayData.products.length < 2 || relayData.isLoading}
              >
                {#each relayData.products as product (product.key)}
                  <option value={product.key}>{product.label}</option>
                {/each}
              </select>
            </label>
            <div class="scale-control">
              <span class="control-label"
                >{markerShape === "circle" ? "円" : "四角形"}の大きさ</span
              >
              <button
                class="step-button scale-smaller"
                aria-label="マーカーを小さく"
                onclick={() => adjustScale(-1)}
                disabled={scale <= MIN_MARKER_SCALE}>−</button
              >
              <input
                aria-label="マーカーのスケール"
                type="number"
                min={MIN_MARKER_SCALE}
                step="1"
                bind:value={scale}
                onblur={() => (scale = normalizeScale(scale))}
              />
              <button
                class="step-button scale-larger"
                aria-label="マーカーを大きく"
                onclick={() => adjustScale(1)}>＋</button
              >
            </div>
            <div class="shape-settings">
              <span class="control-label">形を変える</span>
              <div
                class="shape-control"
                role="group"
                aria-label="地図マーカーの形"
              >
                <button
                  type="button"
                  aria-pressed={markerShape === "circle"}
                  onclick={() => (markerShape = "circle")}>円</button
                >
                <button
                  type="button"
                  aria-pressed={markerShape === "square"}
                  onclick={() => (markerShape = "square")}>四角形</button
                >
              </div>
            </div>
            <div class="month-control">
              <button
                class="step-button month-previous"
                aria-label="前月"
                onclick={() => changeMonth(monthIndex - 1)}
                disabled={monthIndex <= 0}>‹</button
              >
              <input
                aria-label="年月"
                type="range"
                min="0"
                max={Math.max(0, months.length - 1)}
                value={Math.max(0, monthIndex)}
                oninput={(event) =>
                  changeMonth(Number(event.currentTarget.value))}
                disabled={months.length < 2}
              />
              <button
                class="step-button month-next"
                aria-label="次月"
                onclick={() => changeMonth(monthIndex + 1)}
                disabled={monthIndex < 0 || monthIndex >= months.length - 1}
                >›</button
              >
              <strong>{displayMonth(selectedMonth)}</strong>
            </div>
          </fieldset>

          <fieldset class="selection-panel ui-panel">
            <legend>産地別の報告値と消費地域別内訳</legend>
            <div class="selection-heading">
              <div>
                <p class="eyebrow">
                  {displayMonth(selectedMonth)} · 報告値合計
                </p>
                <h3>{selectedPrefecture || "産地を選択"}</h3>
                <strong class="total">{valueLabel(selectedObservation)}</strong>
              </div>
            </div>
            {#if selectedObservation}
              <p class="detail-note">
                合計しているのは、主要消費地域9区分のうち数値が報告された区分です。
                {#if excludedCategoryCounts.length > 0}
                  合計対象外の区分数：
                  {#each excludedCategoryCounts as category, index (category.label)}
                    {#if index > 0}、{/if}{category.label}（{category.count}区分）
                  {/each}。
                {/if}
              </p>
              {#if relayData.isDetailLoading}
                <p role="status" class="detail-note">内訳を読み込んでいます…</p>
              {:else if relayData.detailError}
                <p class="error" role="alert">
                  {relayData.detailError.message}
                </p>
                <button type="button" onclick={retryDetail}
                  >内訳を再読み込み</button
                >
              {:else if !detailContributions}
                <p class="detail-note">
                  地図で都道府県を選ぶと、消費地域別の内訳を表示します。
                </p>
              {:else}
                <ul class="contributions">
                  {#each detailContributions as cell (cell.consumer_region_code)}
                    {@const display = contributionDisplay(cell)}
                    <li>
                      <span class="region-name"
                        >{cell.consumer_region_name}</span
                      >
                      <strong>
                        {display.value}
                        {#if display.note}<small>{display.note}</small>{/if}
                      </strong>
                    </li>
                  {/each}
                </ul>
              {/if}
            {:else if selectedPrefecture}
              <p class="detail-note">この出荷元のデータはありません。</p>
            {:else}
              <p class="detail-note">
                地図で都道府県を選ぶと、報告値と消費地域別の内訳を表示します。
              </p>
            {/if}
          </fieldset>

          <fieldset class="sankey-panel ui-panel">
            <legend>産地と消費地域の Sankey 図</legend>
            {#if !relayData.sankeyOpen}
              <button
                type="button"
                onclick={() => relayData.openSankey()}
                aria-expanded="false"
                aria-controls="relay-sankey">図を表示</button
              >
            {:else}
              <h2>
                {selectedProductLabel} · {displayMonth(
                  selectedMonth,
                )}の産地都道府県別の卸売数量
              </h2>
              <button
                type="button"
                onclick={() => relayData.closeSankey()}
                aria-expanded="true"
                aria-controls="relay-sankey">Sankey を閉じる</button
              >
              <div id="relay-sankey" aria-busy={relayData.isSankeyLoading}>
                {#if relayData.isSankeyLoading}
                  <p role="status">
                    産地内訳を読み込んでいます…（{relayData.sankeyCompleted} / 47
                    都道府県）
                  </p>
                {:else if relayData.sankeyFailures.length}
                  <p class="error" role="alert">
                    データを取得できませんでした（{relayData.sankeyFailures
                      .length} 都道府県）。再度お試しください。
                  </p>
                  <button type="button" onclick={() => relayData.openSankey()}
                    >失敗したデータを再取得</button
                  >
                {:else if sankeyData}
                  <VegetableSankey data={sankeyData} />
                {/if}
              </div>
            {/if}
          </fieldset>

          {@render relayReference()}
        </section>
      </div>
    {:else if dataError}
      <LoadState
        state="error"
        message={dataError}
        onretry={reloadPage}
        retryLabel="ページを再読み込み"
      />
    {/if}
    {#if !aggregation && !relayData.isLoading}{@render relayReference()}{/if}
  </main>
</div>

<style>
  .visualization-page--relay {
    --control-size: var(--control-height);
    --scale-value-width: 4rem;
    --workspace-columns: minmax(0, 1.2fr) minmax(24rem, 1fr);
    --workspace-areas: "details map";
  }
  .workspace {
    overflow: hidden;
  }
  .map-column {
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
    grid-area: map;
  }
  .map-view {
    flex: 1;
    min-height: 0;
    position: relative;
  }
  .map-loading {
    position: absolute;
    inset-block-start: var(--space-3);
    inset-inline-start: var(--space-3);
    z-index: 1000;
    pointer-events: none;
  }
  .map-loading :global(.load-state) {
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-color);
    box-shadow: var(--tooltip-shadow);
  }
  .map-heading {
    padding: var(--space-4) var(--space-6);
    background: var(--surface-color);
    border-bottom: var(--border-default);
  }
  .map-heading h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  .map-heading p {
    margin: var(--space-1) 0 0;
    font-size: 0.75rem;
    color: var(--text-muted-color);
  }
  .details-column h2,
  .selection-heading h3 {
    font-size: 1rem;
  }

  .controls,
  .map-heading {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .map-heading {
    justify-content: space-between;
    margin-bottom: 0;
  }
  .controls {
    background: var(--surface-color);
    width: auto;
    max-width: 100%;
    flex-wrap: wrap;
    flex-direction: row;
    align-items: center;
    gap: var(--space-6);
    padding: var(--space-3);
  }
  .controls label {
    display: grid;
    gap: var(--space-1);
  }
  .vegetable-control {
    align-self: center;
  }
  .month-control,
  .scale-control {
    display: grid;
    align-items: center;
    width: 100%;
    gap: var(--space-2);
  }
  .month-control {
    margin-bottom: var(--space-3);
    grid-template-columns: var(--control-size) minmax(0, 1fr) var(
        --control-size
      );
    min-width: 0;
    grid-template-areas:
      "date date date"
      "previous range next";
  }
  .month-control > .month-previous {
    grid-area: previous;
  }
  .month-control input[type="range"] {
    grid-area: range;
    min-width: 0;
  }
  .month-control > .month-next {
    grid-area: next;
  }
  .step-button {
    width: var(--control-size);
    height: var(--control-height);
    padding: 0;
    font-size: 1.25rem;
    line-height: 1;
  }
  .month-control > strong {
    grid-area: date;
    text-align: left;
    white-space: nowrap;
  }
  .scale-control {
    width: auto;
    grid-template-columns: var(--control-size) var(--scale-value-width) var(
        --control-size
      );
    grid-template-areas:
      "label label label"
      "smaller value larger";
  }
  .scale-control > .control-label {
    grid-area: label;
  }
  .shape-control {
    display: flex;
    gap: var(--space-1);
  }
  .shape-settings {
    display: grid;
    align-content: start;
    gap: var(--space-1);
  }
  .shape-control button {
    min-height: var(--control-height);
    padding-inline: var(--space-3);
  }
  .scale-control > .scale-smaller {
    grid-area: smaller;
  }
  .scale-control > input {
    grid-area: value;
  }
  .scale-control > .scale-larger {
    grid-area: larger;
  }
  .scale-control input {
    box-sizing: border-box;
    width: var(--scale-value-width);
    padding-inline: var(--space-1);
  }
  @media (max-width: 360px) {
    .visualization-page--relay {
      --control-size: 2.25rem;
      --scale-value-width: 3.25rem;
    }
    .controls {
      gap: var(--space-1);
    }
    .month-control,
    .scale-control {
      gap: var(--space-1);
    }
  }
  .details-column > fieldset {
    min-inline-size: 0;
    margin: 0 0 var(--space-4);
  }
  .details-column > fieldset > legend {
    padding-inline: var(--space-1);
    font-weight: 600;
  }
  .total {
    display: block;
    margin-top: var(--space-1);
  }
  .contributions {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .contributions li {
    padding: var(--space-2);
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-color);
  }
  @media (max-width: 50rem) {
    .workspace {
      overflow: visible;
    }
    .map-column {
      height: 56dvh;
      min-height: 340px;
    }
    .details-column {
      border-bottom: 0;
    }

    .controls {
      width: auto;
      max-width: none;
      align-items: stretch;
      flex-direction: column;
      gap: var(--space-3);
    }
    .scale-control {
      width: 100%;
    }
    .contributions {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
</style>
