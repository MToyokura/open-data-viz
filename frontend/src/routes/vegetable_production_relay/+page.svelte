<script lang="ts">
  import type { PrefectureMonth } from "./constants/types";
  import VegetableSankey from "./VegetableSankey.svelte";
  import { aggregateSankey } from "./utils/aggregateSankey";
  import RelayMap from "./RelayMap.svelte";
  import { useRelayData } from "./utils/useRelayData.svelte";

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
  let scale = $state(5);

  let yearData = $derived(
    years.find((item) => item.year === Number(selectedMonth.slice(0, 4))),
  );
  let monthIndex = $derived(months.indexOf(selectedMonth));
  let prefectures = $derived(
    yearData?.months.filter((item) => item.month === selectedMonth) ?? [],
  );
  // here
  let observations = $derived(
    Object.fromEntries(
      prefectures.map((item) => [item.consignor_prefecture_name, item]),
    ),
  );
  let sankeyData = $derived(
    relayData.sankeyDetails.length === 47
      ? aggregateSankey(relayData.sankeyDetails, selectedMonth, prefectures)
      : undefined,
  );
  let selectedObservation = $derived(observations[selectedPrefecture]);
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
  // here
  let detailRows = $derived.by(() => {
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

  function changeMonth(index: number) {
    if (index >= 0 && index < months.length) selectedMonth = months[index];
  }
  function selectPrefecture(name: string) {
    selectedPrefecture = name;
    const code = observations[name]?.consignor_prefecture_code;
    if (code) void relayData.selectPrefectureDetail(code);
  }

  function retryDetail() {
    const code = selectedObservation?.consignor_prefecture_code;
    if (code) void relayData.selectPrefectureDetail(code);
  }
  function adjustScale(amount: number) {
    scale = Math.max(1, Number((scale + amount).toFixed(1)));
  }
  const quantityFormatter = new Intl.NumberFormat("ja-JP", {
    maximumFractionDigits: 2,
  });
  function formatValue(value: string) {
    return quantityFormatter.format(Number(value));
  }
  function valueLabel(row: PrefectureMonth | undefined) {
    if (!row) return "値なし";
    if (row.reported_cells === 0) {
      return row.below_display_unit_cells > 0
        ? `数値の報告なし（表示単位未満 ${row.below_display_unit_cells} 区分）`
        : "数値の報告なし";
    }
    return `${formatValue(row.reported_value_sum)} ${row.source_unit}`;
  }
  const displayMonth = (month: string) => {
    const [year, number] = month.split("-");
    return `${year}年 ${Number(number)}月`;
  };
</script>

<svelte:head>
  <title>野菜の産地リレー | Open Data Viz</title>
</svelte:head>

<div class="page-shell">
  <main class="page-content">
    <header class="page-heading">
      <div>
        <h1>野菜の産地リレー</h1>
      </div>
    </header>

    {#if dataError}
      <p class="error" role="alert">{dataError}</p>
    {:else if relayData.isLoading}
      <p class="map-loading" role="status">データを読み込んでいます…</p>
    {/if}
    {#if aggregation}
      <section class="controls ui-panel ui-panel--subtle" aria-label="表示設定">
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
          <span class="control-label">円の大きさ</span>
          <button
            class="step-button scale-smaller"
            aria-label="円を小さく"
            onclick={() => adjustScale(-1)}
            disabled={scale <= 1}>−</button
          >
          <input
            aria-label="円のスケール"
            type="number"
            min="1"
            step="1"
            bind:value={scale}
            onblur={() => (scale = Math.max(1, Number(scale) || 5))}
          />
          <button
            class="step-button scale-larger"
            aria-label="円を大きく"
            onclick={() => adjustScale(1)}>＋</button
          >
        </div>
      </section>

      <div class="map-heading">
        <div>
          <h2>
            {selectedProductLabel} · {displayMonth(selectedMonth)}の出荷数量
          </h2>
          <p>円の面積は報告値の合計に比例します</p>
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
          oninput={(event) => changeMonth(Number(event.currentTarget.value))}
          disabled={months.length < 2}
        />
        <button
          class="step-button month-next"
          aria-label="次月"
          onclick={() => changeMonth(monthIndex + 1)}
          disabled={monthIndex < 0 || monthIndex >= months.length - 1}>›</button
        >
        <strong>{displayMonth(selectedMonth)}</strong>
      </div>
      {#if mapError}
        <p class="error" role="alert">{mapError}</p>
      {:else if !geojson}
        <p class="map-loading" role="status">地図を読み込んでいます…</p>
      {:else}
        <RelayMap
          {geojson}
          {observations}
          sourceUnit={yearData?.source_unit ?? null}
          {scale}
          onSelect={selectPrefecture}
        />
      {/if}

      <section
        class="selection-panel ui-panel"
        aria-label="選択した出荷元の詳細"
      >
        <div class="selection-heading">
          <div>
            <p class="eyebrow">{displayMonth(selectedMonth)} · 報告値合計</p>
            <h2>{selectedPrefecture || "産地を選択"}</h2>
            <strong class="total">{valueLabel(selectedObservation)}</strong>
          </div>
        </div>
        <!-- here -->
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
          <!-- here -->
          {#if relayData.isDetailLoading}
            <p role="status" class="detail-note">内訳を読み込んでいます…</p>
          {:else if relayData.detailError}
            <p class="error" role="alert">{relayData.detailError.message}</p>
            <button type="button" onclick={retryDetail}>内訳を再読み込み</button
            >
          {:else if !detailRows}
            <p class="detail-note">
              地図で都道府県を選ぶと、消費地域別の内訳を表示します。
            </p>
          {:else}
            <ul class="contributions">
              {#each detailRows as cell (cell.consumer_region_code)}
                <li>
                  <span class="region-name">{cell.consumer_region_name}</span>
                  <strong
                    >{#if cell.value_kind === "reported"}{formatValue(
                        cell.raw_value,
                      )}{:else if cell.value_kind === "below_display_unit"}0 <small
                        >表示単位未満</small
                      >{:else if cell.value_kind === "unknown_or_uncollected"}… <small
                        >不詳・調査なし</small
                      >{:else}— <small>該当なし</small>{/if}</strong
                  >
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
      </section>

      <section
        class="sankey-panel ui-panel"
        aria-label="産地と消費地域の Sankey 図"
      >
        {#if !relayData.sankeyOpen}
          <button
            type="button"
            onclick={() => relayData.openSankey()}
            aria-expanded="false"
            aria-controls="relay-sankey">Sankey を表示</button
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
                産地内訳を読み込んでいます…（{relayData.sankeyCompleted} / 47 都道府県）
              </p>
            {:else if relayData.sankeyFailures.length}
              <p class="error" role="alert">
                データを取得できませんでした（{relayData.sankeyFailures.length} 都道府県）。再度お試しください。
              </p>
              <button type="button" onclick={() => relayData.openSankey()}
                >失敗したデータを再取得</button
              >
            {:else if sankeyData}
              <VegetableSankey data={sankeyData} />
            {/if}
          </div>
        {/if}
      </section>

      <section class="notes">
        <h2>備考</h2>
        <p>
          都道府県または円をクリック・タップすると、地図下にその産地の報告値と内訳を表示します。
          地図上のスクロールで拡大・縮小できます。円の大きさは＋／−ボタンで調整できます。
        </p>
        <p>
          報告値合計は、産地都道府県ごとに、主要消費地域9区分のうち数値が報告された区分だけを合計したものです。
          表示単位未満、該当なし、不詳・調査なしの区分は含めていません。農業生産量や全国計を示す値ではありません。
        </p>
        <p>
          令和4年（2022年）に年間取扱量等調査の対象が中央卸売市場の卸売会社のみに見直されました。年をまたぐ比較にはご注意ください。
        </p>
      </section>

      <section class="reference ui-panel ui-panel--subtle">
        <h2>出典情報等</h2>
        <table class="reference-table ui-table">
          <tbody>
            <tr>
              <th>統計表ID</th>
              <td>
                {#if yearData}
                  <div>{yearData.table_id}</div>
                  <div>
                    <a
                      href={`https://www.e-stat.go.jp/dbview?sid=${yearData.table_id}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      >https://www.e-stat.go.jp/dbview?sid={yearData.table_id}</a
                    >
                  </div>
                {/if}
              </td>
            </tr>
            <tr>
              <th>地図データ</th>
              <td>
                <div>
                  国土交通省「国土数値情報 行政区域データ（N03）」2026年版（<a
                    href="https://creativecommons.org/licenses/by/4.0/deed.ja"
                    target="_blank"
                    rel="noopener noreferrer">CC BY 4.0</a
                  >）
                </div>
                <div>
                  <a
                    href="https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2026.html"
                    target="_blank"
                    rel="noopener noreferrer"
                    >https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2026.html</a
                  >
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </section>
    {/if}
  </main>
</div>

<style>
  .page-shell {
    --control-size: var(--control-height);
    --scale-value-width: 4rem;
  }
  .page-heading,
  .controls,
  .map-heading {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .page-heading,
  .map-heading {
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }
  .controls {
    width: fit-content;
    max-width: 100%;
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
    .page-shell {
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
  .sankey-panel {
    margin-top: var(--space-4);
  }
  .selection-panel {
    margin-top: var(--space-4);
  }
  .total {
    display: block;
    margin-top: var(--space-1);
  }
  .contributions {
    display: grid;
    grid-template-columns: repeat(9, minmax(0, 1fr));
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
  .notes {
    margin: var(--space-8) auto var(--space-4);
    line-height: 1.6;
  }
  .reference {
    margin-top: var(--space-8);
  }
  .reference h2 {
    margin-top: 0;
    margin-bottom: var(--space-4);
  }
  .reference-table {
    margin-top: var(--space-4);
  }
  .reference-table a {
    word-break: break-all;
  }
  @media (max-width: 700px) {
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
