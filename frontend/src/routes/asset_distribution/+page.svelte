<script>
  import { onMount } from "svelte";
  import Spinner from "#lib/Spinner.svelte";
  import DistributionChart from "./DistributionChart.svelte";
  import { fetchAssetDistribution } from "./utils/data.js";

  /** @typedef {import("./utils/data.js").Dimension} Dimension */
  /** @typedef {import("./utils/data.js").ComparisonSet} ComparisonSet */
  /** @typedef {{ key: string, label: string, rows: { code: string, label: string, count: number|null, status: string, raw_value: string, annotations: string[] }[] }} DisplaySeries */

  let data = $state(
    /** @type {import("./utils/data.js").AssetDistribution|null} */ (null),
  );
  let loading = $state(true);
  let error = $state("");
  let selections = $state(/** @type {Record<string, string>} */ ({}));
  let compareDimensionId = $state("");
  let compareSetId = $state("");
  let compareCodes = $state(/** @type {string[]} */ ([]));

  const dimensions = $derived(
    data?.dimensions.toSorted((a, b) => a.order - b.order) ?? [],
  );
  const filterDimensions = $derived(
    dimensions.filter((dimension) => dimension.role === "filter"),
  );
  const assetDimension = $derived(
    dimensions.find((dimension) => dimension.role === "asset_class"),
  );
  const assetCategories = $derived(
    assetDimension?.categories
      .filter((category) => category.kind === "regular")
      .toSorted((a, b) => a.order - b.order) ?? [],
  );
  const comparisonDimension = $derived(
    filterDimensions.find((dimension) => dimension.id === compareDimensionId),
  );
  const comparisonSet = $derived(
    comparisonDimension?.comparison_sets?.find(
      (set) => set.id === compareSetId,
    ),
  );
  const comparisonMode = $derived(
    Boolean(comparisonDimension && comparisonSet),
  );

  /** @param {Record<string, string>} codes */
  function findCombination(codes) {
    return data?.combinations.find((combination) =>
      filterDimensions.every(
        (dimension) =>
          combination.dimensions[dimension.id] === codes[dimension.id],
      ),
    );
  }

  /** @param {Dimension} dimension @param {string} code @param {boolean} [omitComparisonAxis] */
  function candidateStatus(dimension, code, omitComparisonAxis = false) {
    if (!data) return { available: false, reason: "データを読み込めません" };
    const codes = { ...selections, [dimension.id]: code };
    if (
      !comparisonMode ||
      (omitComparisonAxis && dimension.id === compareDimensionId)
    ) {
      const combination = findCombination(codes);
      return combination?.status === "published"
        ? { available: true, reason: "" }
        : {
            available: false,
            reason:
              combination?.reason ?? "この条件の組み合わせはデータにありません",
          };
    }
    const published = [];
    const unavailable = [];
    for (const compareCode of comparisonSet?.category_codes ?? []) {
      const combination = findCombination({
        ...codes,
        [compareDimensionId]: compareCode,
      });
      if (combination?.status === "published") published.push(compareCode);
      else
        unavailable.push(
          combination?.reason ?? "この条件の組み合わせはデータにありません",
        );
    }
    const required = published.length >= 2;
    return {
      available: required,
      reason: required
        ? ""
        : ["比較には公表系列が2つ以上必要です", ...new Set(unavailable)].join(
            "。",
          ),
    };
  }

  /** @param {Dimension} dimension @param {ComparisonSet} set */
  function comparisonSetStatus(dimension, set) {
    const published = set.category_codes.filter(
      (code) =>
        findCombination({ ...selections, [dimension.id]: code })?.status ===
        "published",
    ).length;
    return published >= 2
      ? { available: true, reason: "" }
      : {
          available: false,
          reason: "この条件では比較に必要な公表系列が2つありません",
        };
  }

  const currentCombination = $derived(
    !comparisonMode ? findCombination(selections) : undefined,
  );
  const currentSeries = $derived.by(() => {
    if (
      !data ||
      !assetDimension ||
      (comparisonMode && (!comparisonSet || !comparisonDimension))
    )
      return /** @type {DisplaySeries[]} */ ([]);
    const activeData = data;
    const codes = comparisonMode
      ? (comparisonSet?.category_codes ?? []).filter((code) =>
          compareCodes.includes(code),
        )
      : currentCombination?.status === "published"
        ? [null]
        : [];
    return codes.flatMap((code) => {
      const combination = comparisonMode
        ? findCombination({ ...selections, [compareDimensionId]: code ?? "" })
        : currentCombination;
      if (combination?.status !== "published") return [];
      const categoryLabel = comparisonMode
        ? (comparisonDimension?.categories.find(
            (category) => category.code === code,
          )?.label ??
          code ??
          "")
        : "選択条件";
      const rows = assetCategories.map((category) => {
        const item = activeData.items.find(
          (entry) =>
            entry.combination_id === combination.id &&
            entry.asset_class_code === category.code,
        );
        if (!item)
          throw new Error(
            `階級別データがありません: ${combination.id} / ${category.code}`,
          );
        return {
          code: category.code,
          label: category.label,
          count: item.count,
          status: item.status,
          raw_value: item.raw_value,
          annotations: item.annotations,
        };
      });
      return [{ key: combination.id, label: categoryLabel, rows }];
    });
  });
  const unavailableRows = $derived(
    currentSeries.flatMap((series) =>
      series.rows
        .filter((row) => row.status !== "observed")
        .map((row) => ({
          key: `${series.key}:${row.code}`,
          classLabel: row.label,
          seriesLabel: series.label,
          status: row.status,
          raw: row.raw_value,
          notes: row.annotations,
        })),
    ),
  );
  const noComparableSeries = $derived(
    comparisonMode && currentSeries.length < 2,
  );

  async function loadData() {
    loading = true;
    error = "";
    try {
      data = await fetchAssetDistribution();
      selections = Object.fromEntries(
        data.dimensions
          .filter((dimension) => dimension.role === "filter")
          .map((dimension) => [dimension.id, dimension.default_code ?? ""]),
      );
    } catch (cause) {
      error =
        cause instanceof Error
          ? cause.message
          : "データの読み込みに失敗しました";
    } finally {
      loading = false;
    }
  }

  function startComparison() {
    if (!comparisonDimension || !comparisonSet) return;
    compareCodes = comparisonSet.category_codes.filter(
      (code) => candidateStatus(comparisonDimension, code, true).available,
    );
  }

  /** @param {string} id */
  function setCompareDimension(id) {
    compareDimensionId = id;
    const dimension = filterDimensions.find((item) => item.id === id);
    compareSetId = dimension?.comparison_sets?.[0]?.id ?? "";
    startComparison();
  }

  /** @param {string} id */
  function setCompareSet(id) {
    compareSetId = id;
    startComparison();
  }

  function endComparison() {
    compareDimensionId = "";
    compareSetId = "";
    compareCodes = [];
    if (findCombination(selections)?.status !== "published" && data) {
      selections = Object.fromEntries(
        filterDimensions.map((dimension) => [
          dimension.id,
          dimension.default_code ?? "",
        ]),
      );
    }
  }

  /** @param {string} code */
  function toggleCompareCode(code) {
    compareCodes = compareCodes.includes(code)
      ? compareCodes.filter((item) => item !== code)
      : [...compareCodes, code];
  }

  onMount(() => {
    void loadData();
  });
</script>

<svelte:head>
  <title
    >日本の純資産分布｜属性別・純資産額階級別の世帯数（2024年） | Open Data Viz</title
  >
  <meta
    name="description"
    content="令和6年全国家計構造調査の公表データから、属性別の純資産額階級別世帯数を表示します。"
  />
</svelte:head>

<div class="page-shell">
  <main class="page-content">
    <h1>日本の純資産分布</h1>
    <p>属性別・純資産額階級別の世帯数（2024年）</p>

    {#if loading}<Spinner />
    {:else if error}
      <div class="error" role="alert">
        <p>{error}</p>
        <button onclick={loadData}>再試行</button>
      </div>
    {:else if data && assetDimension}
      <section class="selectors" aria-label="世帯属性の選択">
        {#each filterDimensions as dimension (dimension.id)}
          <label>
            {dimension.label}
            <select
              bind:value={selections[dimension.id]}
              disabled={comparisonMode && dimension.id === compareDimensionId}
            >
              {#each dimension.categories.toSorted((a, b) => a.order - b.order) as category (category.code)}
                {@const availability = candidateStatus(
                  dimension,
                  category.code,
                )}
                <option
                  value={category.code}
                  disabled={!availability.available}
                  title={availability.reason}
                  >{category.label}{availability.available
                    ? ""
                    : `（利用不可: ${availability.reason}）`}</option
                >
              {/each}
            </select>
            {#if comparisonMode && dimension.id === compareDimensionId}<span
                class="hint"
                >比較中はこの軸の単一区分を固定条件に使いません。</span
              >{/if}
          </label>
        {/each}
      </section>

      <section class="comparison ui-panel" aria-label="属性の比較">
        <h2>属性を比較</h2>
        {#if !filterDimensions.some((dimension) => dimension.comparison_sets?.length)}
          <p>このデータには比較可能な区分集合がありません。</p>
        {:else}
          <div class="compare-controls">
            <label
              >比較する属性
              <select
                value={compareDimensionId}
                onchange={(event) =>
                  event.currentTarget.value
                    ? setCompareDimension(event.currentTarget.value)
                    : endComparison()}
              >
                <option value="">比較しない</option>
                {#each filterDimensions.filter((dimension) => dimension.comparison_sets?.length) as dimension (dimension.id)}<option
                    value={dimension.id}>{dimension.label}</option
                  >{/each}
              </select>
            </label>
            {#if comparisonDimension}
              <label
                >区分集合
                <select
                  value={compareSetId}
                  onchange={(event) => setCompareSet(event.currentTarget.value)}
                >
                  {#each comparisonDimension.comparison_sets as set (set.id)}
                    {@const availability = comparisonSetStatus(
                      comparisonDimension,
                      set,
                    )}
                    <option
                      value={set.id}
                      disabled={!availability.available}
                      title={availability.reason}
                      >{set.label}{availability.available
                        ? ""
                        : `（利用不可: ${availability.reason}）`}</option
                    >
                  {/each}
                </select>
              </label>
            {/if}
          </div>
          {#if comparisonSet && comparisonDimension}
            <fieldset class="compare-options">
              <legend>比較する区分</legend>
              {#each comparisonSet.category_codes as code (code)}
                {@const category = comparisonDimension.categories.find(
                  (item) => item.code === code,
                )}
                {@const availability = candidateStatus(
                  comparisonDimension,
                  code,
                  true,
                )}
                <label class:unavailable={!availability.available}>
                  <input
                    type="checkbox"
                    checked={compareCodes.includes(code)}
                    disabled={!availability.available}
                    onchange={() => toggleCompareCode(code)}
                  />
                  {category?.label ?? code}{availability.available
                    ? ""
                    : ` — ${availability.reason}`}
                </label>
              {/each}
            </fieldset>
            {#if noComparableSeries}<p class="hint" role="status">
                比較に必要な公表系列数を満たしていません。区分や他の条件を変更してください。
              </p>{/if}
          {/if}
        {/if}
      </section>

      {#if !comparisonMode && currentCombination?.status === "not_published"}<p
          class="hint"
          role="status"
        >
          この条件は公表されていません: {currentCombination.reason}
        </p>{/if}
      {#if currentSeries.length > 0}
        <h2 class="chart-title">純資産額階級別の世帯数</h2>
        {#if comparisonMode}<p class="hint">
            属性の比較中です。選択軸の単一区分条件は使っていません。
          </p>{/if}
        <DistributionChart
          categories={assetCategories}
          series={currentSeries}
        />
        {#if unavailableRows.length}
          <section class="value-states" aria-label="秘匿・欠損値">
            <h2>数値を表示できない階級</h2>
            <ul>
              {#each unavailableRows as row (row.key)}<li>
                  <strong>{row.classLabel}</strong> — {row.seriesLabel}: {row.status ===
                  "suppressed"
                    ? "秘匿"
                    : "欠損"}{row.raw
                    ? `（原表記: ${row.raw}）`
                    : ""}{#if row.notes.length}。{row.notes.join("、")}{/if}
                </li>{/each}
            </ul>
          </section>
        {/if}
      {:else if !comparisonMode && currentCombination?.status !== "not_published"}
        <p class="hint" role="status">表示できる公表系列がありません。</p>
      {/if}

      <section class="reference ui-panel ui-panel--subtle">
        <h2>データについて</h2>
        <p>
          調査年: {String(data.metadata.year ?? "")}年（公表日: {String(
            data.metadata.published_at ?? ""
          )}）。対象: {String(
            data.metadata.area ?? "全国",
          )}。表章項目: {String(data.metadata.measure ?? "世帯数")}（{String(
            data.metadata.unit ?? "世帯",
          )}）。階級の定義: {String(
            data.metadata.asset_basis ?? "純資産総額",
          )}。
        </p>
        <p>
          世帯数は調査世帯の単純な回答数ではなく、集計用乗率を使った推計値です。純資産総額が0またはマイナスの世帯は「25万円未満」に含まれ、資産額不詳の世帯は平均にのみ含まれるため、金額階級の棒には含まれません。階級の境界と開放端は統計表の公式表記を使っています。
        </p>
        <p>
          統計表の「-」は該当数字なし、「0」は表章単位未満、「X」は秘匿を表します。これらを0世帯として描かず、数値が得られない階級として表示します。API応答に単位属性がないため、表題の「世帯数」に基づき単位を「世帯」としています。公表値は整数に丸められており、階級別の合計と平均（資産額不詳を除く）が1世帯ずれる場合があります。
        </p>
        <h2>出典</h2>
        <p>
          総務省統計局「令和6年全国家計構造調査」表7-131-1-1「資産額階級別世帯数」
        </p>
        <p>
          <a
            href="https://www.e-stat.go.jp/dbview?sid=0004056120"
            target="_blank"
            rel="noopener noreferrer">統計表ID 0004056120（e-Stat）</a
          >
        </p>
      </section>
    {/if}
  </main>
</div>

<style>
  h1 {
    font-size: 2rem;
  }
  .selectors {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 14rem), 1fr));
    gap: var(--space-4);
    margin: var(--space-6) 0;
  }
  .selectors label,
  .compare-controls label {
    display: grid;
    align-content: start;
    gap: var(--space-2);
  }
  select {
    min-width: 0;
  }
  .comparison {
    margin: var(--space-6) 0;
  }
  .comparison h2,
  .reference h2,
  .value-states h2 {
    margin: 0 0 var(--space-3);
    font-size: 1.1rem;
  }
  .compare-controls {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 14rem), 1fr));
    gap: var(--space-4);
  }
  .compare-options {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3) var(--space-6);
    margin: var(--space-4) 0 0;
    padding: var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
  }
  .compare-options label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .unavailable {
    color: #666;
  }
  .hint {
    color: #555;
  }
  .chart-title {
    margin-top: var(--space-8);
  }
  .value-states {
    margin: var(--space-6) 0;
  }
  .value-states ul {
    padding-left: 1.4rem;
  }
  .reference {
    margin-top: var(--space-8);
  }
  .reference p:last-child {
    margin-bottom: 0;
  }
  .error {
    color: var(--danger-color);
  }
  @media (max-width: 42rem) {
    .compare-options {
      display: grid;
    }
  }
</style>
