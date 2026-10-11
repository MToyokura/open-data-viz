<script lang="ts">
  import type { PrefectureMonth } from "./constants/types";
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
  let selectedMonth = $state("2023-01");
  let selectedPrefecture = $state("北海道");
  let scale = $state(5);

  let yearData = $derived(
    years.find((item) => item.year === Number(selectedMonth.slice(0, 4))) ??
      years[0],
  );
  let monthIndex = $derived(months.indexOf(selectedMonth));
  let prefectures = $derived(
    yearData?.months.filter((item) => item.month === selectedMonth) ?? [],
  );
  let observations = $derived(
    Object.fromEntries(
      prefectures.map((item) => [item.consignor_prefecture_name, item]),
    ),
  );
  let selectedObservation = $derived(
    observations[selectedPrefecture] as PrefectureMonth | undefined,
  );

  function changeMonth(index: number) {
    if (index >= 0 && index < months.length) selectedMonth = months[index];
  }
  function selectPrefecture(name: string) {
    selectedPrefecture = name;
  }
  function adjustScale(amount: number) {
    scale = Math.min(20, Math.max(1, Number((scale + amount).toFixed(1))));
  }
  function formatValue(value: string) {
    return new Intl.NumberFormat("ja-JP", { maximumFractionDigits: 2 }).format(
      Number(value),
    );
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
  <title>野菜の産地別卸売数量リレー | Open Data Viz</title>
</svelte:head>

<div class="relay-page">
  <main>
    <header class="page-heading">
      <div>
        <p class="eyebrow">青果物卸売市場調査</p>
        <h1>野菜の産地別卸売数量リレー</h1>
      </div>
    </header>

    {#if dataError}
      <p class="error" role="alert">{dataError}</p>
    {:else if relayData.isLoading}
      <p class="map-loading" role="status">データを読み込んでいます…</p>
    {/if}
    {#if aggregation?.mapping_warnings?.length}
      <aside class="data-notice" aria-label="月対応の注意" role="note">
        <strong>一部の月は同じ値の地域区分に対応します</strong>
        <ul>
          {#each aggregation.mapping_warnings as warning (warning)}
            <li>{warning}</li>
          {/each}
        </ul>
      </aside>
    {/if}
    {#if aggregation?.status === "incomplete"}
      <aside class="data-notice" aria-label="データの欠測状況" role="note">
        <strong>一部に不完全な値があります</strong>
        <p>
          表示単位未満、該当なし、不詳または調査なしの値は数値合計に含めず、地域ごとの詳細に状態を表示します。
        </p>
      </aside>
    {/if}
    {#if aggregation}
      <section class="controls" aria-label="表示設定">
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
        <div class="month-control">
          <span class="control-label">年月</span>
          <button
            class="step-button"
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
            class="step-button"
            aria-label="次月"
            onclick={() => changeMonth(monthIndex + 1)}
            disabled={monthIndex < 0 || monthIndex >= months.length - 1}
            >›</button
          >
          <strong>{displayMonth(selectedMonth)}</strong>
        </div>
        <div class="scale-control">
          <span class="control-label">円の大きさ</span>
          <button
            class="step-button"
            aria-label="円を小さく"
            onclick={() => adjustScale(-0.5)}
            disabled={scale <= 1}>−</button
          >
          <input
            aria-label="円のスケール"
            type="number"
            min="1"
            max="20"
            step="0.5"
            bind:value={scale}
            onblur={() =>
              (scale = Math.min(20, Math.max(1, Number(scale) || 5)))}
          />
          <button
            class="step-button"
            aria-label="円を大きく"
            onclick={() => adjustScale(0.5)}
            disabled={scale >= 20}>＋</button
          >
          <small>基準 5</small>
        </div>
      </section>

      <div class="map-heading">
        <div>
          <h2>
            {selectedProductLabel} · {displayMonth(selectedMonth)}の出荷元
          </h2>
          <p>円の面積は報告値の合計に比例します</p>
        </div>
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

      <section class="selection-panel" aria-label="選択した出荷元の詳細">
        <div class="selection-heading">
          <div>
            <p class="eyebrow">{displayMonth(selectedMonth)} · 報告値合計</p>
            <h2>{selectedPrefecture}</h2>
          </div>
          <strong class="total">{valueLabel(selectedObservation)}</strong>
        </div>
        {#if selectedObservation}
          <p class="detail-note">
            主要消費地域9区分の数値を合計しています。表示単位未満 {selectedObservation.below_display_unit_cells}
            区分、該当なし {selectedObservation.no_fact_cells} 区分、不詳・調査なし
            {selectedObservation.unknown_or_uncollected_cells} 区分は含みません。
          </p>
          <ul class="contributions">
            {#each selectedObservation.contributions as cell (cell.consumer_region_code)}
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
        {:else}
          <p class="detail-note">この出荷元のデータはありません。</p>
        {/if}
      </section>

      <section class="notes">
        <h2>備考</h2>
        <p>
          都道府県をマウスでホバー、またはタップすると報告値を確認できます。円の大きさは操作盤で変更できます。
        </p>
        <p>
          数値は報告された産地都道府県ごとに、主要消費地域9区分への卸売数量を合計したものです。農業生産量や全国計ではありません。
        </p>
        <p>
          令和4年（2022年）に年間取扱量等調査の対象が中央卸売市場の卸売会社のみに見直されました。年をまたぐ比較にはご注意ください。
        </p>
        <p>
          統計表: e-Stat {yearData?.table_id} · 単位: {yearData?.source_unit} · 地図:
          国土交通省 国土数値情報 N03
        </p>
      </section>
    {/if}
  </main>
</div>

<style>
  .relay-page {
    min-height: 100vh;
    background: #fff;
    color: #28312c;
  }
  main {
    max-width: 1280px;
    margin: auto;
    padding: 1.5rem;
  }
  .page-heading,
  .controls,
  .month-control,
  .scale-control,
  .map-heading,
  .selection-heading {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .page-heading,
  .map-heading,
  .selection-heading {
    justify-content: space-between;
  }
  .page-heading,
  .map-heading {
    margin-bottom: 1rem;
  }
  .controls {
    flex-wrap: wrap;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    background: white;
  }
  .controls label {
    display: grid;
    gap: 0.25rem;
  }
  .month-control {
    flex: 1;
    min-width: 260px;
  }
  .month-control input[type="range"] {
    flex: 1;
  }
  .scale-control input {
    width: 4rem;
  }
  .data-notice {
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    border-left: 4px solid #999;
    background: #f5f5f5;
  }
  .data-notice p {
    margin-bottom: 0;
  }
  .selection-panel {
    margin-top: 1rem;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    background: white;
  }
  .contributions {
    display: grid;
    grid-template-columns: repeat(9, minmax(0, 1fr));
    gap: 0.5rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .contributions li {
    padding: 0.5rem;
    border: 1px solid #eee;
  }
  .notes {
    max-width: 850px;
    margin: 2rem auto 1rem;
    line-height: 1.6;
  }
  @media (max-width: 700px) {
    main {
      padding: 1rem;
    }
    .controls,
    .month-control {
      align-items: stretch;
      flex-direction: column;
    }
    .contributions {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }
</style>
