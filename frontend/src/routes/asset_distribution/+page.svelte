<script>
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import LicenseLink from "#lib/LicenseLink.svelte";
  import { onMount } from "svelte";
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import Spinner from "#lib/Spinner.svelte";
  import DistributionChart from "./DistributionChart.svelte";
  import {
    fetchAssetDistribution,
    fetchCombinationItems,
    findCombination,
    getDisplayAssetCategories,
  } from "./utils/data.js";

  /** @typedef {import("./utils/data.js").Dimension} Dimension */
  /** @typedef {import("./utils/data.js").ComparisonSet} ComparisonSet */
  /** @typedef {{ key: string, label: string, rows: { code: string, label: string, count: number|null, status: string, raw_value: string, annotations: string[] }[] }} DisplaySeries */

  let data = $state.raw(
    /** @type {import("./utils/data.js").AssetDistribution|null} */ (null),
  );
  let loading = $state(true);
  let error = $state("");
  let selections = $state(/** @type {Record<string, string>} */ ({}));
  let compareDimensionId = $state("");
  let compareSetId = $state("");
  let compareCodes = $state(/** @type {string[]} */ ([]));
  let normalCombinations = $state.raw(
    /** @type {import("./utils/data.js").Combination[]} */ ([]),
  );
  let seriesItems = $state.raw(
    /** @type {Record<string, import("./utils/data.js").DistributionItem[]>} */ ({}),
  );
  let seriesLoading = $state(false);
  let seriesError = $state("");
  let seriesLoadGeneration = 0;
  let orientation = $state(
    /** @type {"vertical"|"horizontal"} */ ("horizontal"),
  );
  let isMobile = $state(false);
  let isSelectorOpen = $state(true);
  let isComparisonOpen = $state(true);

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
    data
      ? getDisplayAssetCategories(data).map((category) => ({
          ...category,
          label: displayCategoryLabel(category.label),
        }))
      : [],
  );

  /** @param {string} label */
  function displayCategoryLabel(label) {
    return label.replace(/[（(]再掲[）)]/g, "");
  }
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
  function combinationFor(codes) {
    return data ? findCombination(data, codes) : undefined;
  }

  /** @param {import("./utils/data.js").Combination} combination */
  function displaySeriesLabel(combination) {
    if (comparisonMode) {
      const code = combination.dimensions[compareDimensionId];
      return (
        comparisonDimension?.categories.find(
          (category) => category.code === code,
        )?.label ?? code
      );
    }
    return filterDimensions
      .map((dimension) => {
        const code = combination.dimensions[dimension.id];
        return (
          dimension.categories.find((category) => category.code === code)
            ?.label ?? code
        );
      })
      .filter(Boolean)
      .join("・");
  }

  /** @param {import("./utils/data.js").Combination|undefined} combination @returns {combination is import("./utils/data.js").Combination} */
  function isPublishedCombination(combination) {
    return combination?.status === "published";
  }

  /** @param {Dimension} dimension @param {string} code @param {boolean} [omitComparisonAxis] */
  function candidateStatus(dimension, code, omitComparisonAxis = false) {
    if (!data) return { available: false, reason: "データを読み込めません" };
    const codes = { ...selections, [dimension.id]: code };
    if (
      !comparisonMode ||
      (omitComparisonAxis && dimension.id === compareDimensionId)
    ) {
      const combination = combinationFor(codes);
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
      const combination = combinationFor({
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
        combinationFor({ ...selections, [dimension.id]: code })?.status ===
        "published",
    ).length;
    return published >= 2
      ? { available: true, reason: "" }
      : {
          available: false,
          reason: "この条件では比較に必要な公表系列が2つありません",
        };
  }

  const activeCombinations = $derived.by(() => {
    if (!data)
      return /** @type {import("./utils/data.js").Combination[]} */ ([]);
    if (!comparisonMode) return normalCombinations;
    const combinations = (comparisonSet?.category_codes ?? [])
      .filter((code) => compareCodes.includes(code))
      .map((code) =>
        combinationFor({ ...selections, [compareDimensionId]: code }),
      )
      .filter(isPublishedCombination);
    return [
      ...new Map(
        combinations.map((combination) => [combination.id, combination]),
      ).values(),
    ];
  });
  const currentSeries = $derived.by(() => {
    if (
      !data ||
      !assetDimension ||
      (comparisonMode && (!comparisonSet || !comparisonDimension))
    )
      return /** @type {DisplaySeries[]} */ ([]);
    if (activeCombinations.some((combination) => !seriesItems[combination.id]))
      return /** @type {DisplaySeries[]} */ ([]);
    return activeCombinations.map((combination) => {
      const code = combination.dimensions[compareDimensionId];
      const categoryLabel = comparisonMode
        ? (comparisonDimension?.categories.find(
            (category) => category.code === code,
          )?.label ??
          code ??
          "")
        : "選択条件";
      const items = seriesItems[combination.id];
      if (!items)
        throw new Error(`階級別データがありません: ${combination.id}`);
      const itemsByCode = new Map(
        items.map((item) => [item.asset_class_code, item]),
      );
      const rows = assetCategories.map((category) => {
        const item = itemsByCode.get(category.code);
        if (!item)
          throw new Error(
            `階級別データがありません: ${combination.id} / ${category.code}`,
          );
        return {
          code: category.code,
          label: displayCategoryLabel(category.label),
          count: item.count,
          status: item.status,
          raw_value: item.raw_value,
          annotations: item.annotations,
        };
      });
      return { key: combination.id, label: categoryLabel, rows };
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
    comparisonMode && activeCombinations.length < 2,
  );

  async function loadData() {
    loading = true;
    error = "";
    seriesError = "";
    seriesItems = {};
    normalCombinations = [];
    try {
      data = await fetchAssetDistribution();
      selections = Object.fromEntries(
        data.dimensions
          .filter((dimension) => dimension.role === "filter")
          .map((dimension) => [dimension.id, dimension.default_code ?? ""]),
      );
      const initialCombination = combinationFor(selections);
      normalCombinations =
        initialCombination?.status === "published" ? [initialCombination] : [];
    } catch (cause) {
      error =
        cause instanceof Error
          ? cause.message
          : "データの読み込みに失敗しました";
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    const manifest = data;
    const combinations = activeCombinations;
    const generation = ++seriesLoadGeneration;
    seriesError = "";
    const missing = combinations.filter(
      (combination) => !seriesItems[combination.id],
    );
    if (!manifest || missing.length === 0) {
      seriesLoading = false;
      return;
    }
    seriesLoading = true;
    void Promise.all(
      missing.map(async (combination) => [
        combination.id,
        await fetchCombinationItems(manifest, combination.id),
      ]),
    )
      .then((loaded) => {
        if (generation !== seriesLoadGeneration) return;
        seriesItems = { ...seriesItems, ...Object.fromEntries(loaded) };
      })
      .catch((cause) => {
        if (generation !== seriesLoadGeneration) return;
        seriesError =
          cause instanceof Error
            ? cause.message
            : "系列データの読み込みに失敗しました";
      })
      .finally(() => {
        if (generation === seriesLoadGeneration) seriesLoading = false;
      });
  });

  function retrySeries() {
    const activeIds = new Set(
      activeCombinations.map((combination) => combination.id),
    );
    seriesItems = Object.fromEntries(
      Object.entries(seriesItems).filter(([id]) => !activeIds.has(id)),
    );
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
    if (combinationFor(selections)?.status !== "published" && data) {
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

  /** @param {string} code */
  function removeComparisonCode(code) {
    compareCodes = compareCodes.filter((item) => item !== code);
  }

  /** @param {import("./utils/data.js").Combination} combination */
  function removeSeries(combination) {
    if (comparisonMode) {
      removeComparisonCode(combination.dimensions[compareDimensionId]);
      return;
    }
    normalCombinations = normalCombinations.filter(
      (item) => item.id !== combination.id,
    );
  }

  /** @param {string} dimensionId @param {string} code */
  function changeSelection(dimensionId, code) {
    selections = { ...selections, [dimensionId]: code };
    if (!comparisonMode) {
      const combination = combinationFor(selections);
      if (
        combination?.status === "published" &&
        !normalCombinations.some((item) => item.id === combination.id)
      ) {
        normalCombinations = [...normalCombinations, combination];
      }
    }
  }

  function clearSeries() {
    if (comparisonMode) compareCodes = [];
    else normalCombinations = [];
  }

  onMount(() => {
    const mobileQuery = window.matchMedia("(max-width: 50rem)");
    const updateViewport = () => {
      isMobile = mobileQuery.matches;
      isSelectorOpen = !isMobile;
      isComparisonOpen = !isMobile || activeCombinations.length === 0;
    };
    updateViewport();
    mobileQuery.addEventListener("change", updateViewport);
    void loadData();
    return () => mobileQuery.removeEventListener("change", updateViewport);
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

<div class="visualization-page visualization-page--chart-first">
  <main class="page-content">
    <WorkspaceHeader title="日本の純資産分布" />

    <div class="workspace">
      <aside class="details-column" aria-label="比較条件">
        {#if loading}<Spinner />
        {:else if error}
          <div class="error" role="alert">
            <p>{error}</p>
            <button onclick={loadData}>再試行</button>
          </div>
        {:else if data && assetDimension}
          <section class="control-panel" aria-label="世帯属性の選択">
            <details class="selector-panel" bind:open={isSelectorOpen}>
              <summary>世帯属性・比較条件</summary>
              <div class="selector-panel-content">
                <fieldset class="series-condition-list">
                  <legend>系列の条件</legend>
                  <p class="intro">
                    属性別・純資産額階級別の世帯数（2024年）。条件を変えると、その条件の系列がグラフに追加されます。
                  </p>
                  <section class="selectors" aria-label="世帯属性">
                    {#each filterDimensions as dimension (dimension.id)}
                      <label>
                        {dimension.label}
                        <select
                          value={selections[dimension.id]}
                          disabled={comparisonMode &&
                            dimension.id === compareDimensionId}
                          onchange={(event) =>
                            changeSelection(
                              dimension.id,
                              event.currentTarget.value,
                            )}
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
                        {#if comparisonMode && dimension.id === compareDimensionId}
                          <span class="hint"
                            >比較中はこの軸の単一区分条件を使いません。</span
                          >
                        {/if}
                      </label>
                    {/each}
                  </section>
                  <section class="comparison-controls" aria-label="属性比較">
                    <label>
                      比較する属性
                      <select
                        value={compareDimensionId}
                        onchange={(event) =>
                          event.currentTarget.value
                            ? setCompareDimension(event.currentTarget.value)
                            : endComparison()}
                      >
                        <option value="">比較しない</option>
                        {#each filterDimensions.filter((dimension) => dimension.comparison_sets?.length) as dimension (dimension.id)}
                          <option value={dimension.id}>{dimension.label}</option
                          >
                        {/each}
                      </select>
                    </label>
                    {#if comparisonDimension}
                      <label>
                        区分集合
                        <select
                          value={compareSetId}
                          onchange={(event) =>
                            setCompareSet(event.currentTarget.value)}
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
                  </section>
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
                    {#if noComparableSeries}
                      <p class="hint" role="status">
                        比較に必要な公表系列数を満たしていません。区分や他の条件を変更してください。
                      </p>
                    {/if}
                  {/if}
                </fieldset>
                <label class="orientation">
                  グラフの向き
                  <select bind:value={orientation}>
                    <option value="horizontal">横棒</option>
                    <option value="vertical">縦棒</option>
                  </select>
                </label>
              </div>
            </details>
          </section>
          <section
            class="comparison-panel"
            aria-label={comparisonMode ? "比較中のデータ" : "表示中の系列"}
          >
            <details class="added-items" bind:open={isComparisonOpen}>
              <summary
                ><h2>
                  {comparisonMode ? "比較中" : "表示中の系列"}
                  <span>{activeCombinations.length}系列</span>
                </h2></summary
              >
              <fieldset class="selected-list">
                <legend>選択中一覧</legend>
                {#if activeCombinations.length}
                  <ul>
                    {#each activeCombinations as combination (combination.id)}
                      {@const label = displaySeriesLabel(combination)}
                      <li>
                        <span>{label}</span>
                        <button
                          type="button"
                          class="icon-button"
                          aria-label="{label}を選択中一覧から削除"
                          onclick={() => removeSeries(combination)}
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
                  <p class="empty-comparison">
                    比較できる公表系列がありません。
                  </p>
                {/if}
              </fieldset>
              <button
                class="reset-button"
                onclick={clearSeries}
                disabled={activeCombinations.length === 0}>すべて消去</button
              >
            </details>
          </section>
        {/if}

        {#if data}
          <ReferencePanel title="データについて" marginTop="var(--space-4)">
            <table class="reference-table ui-table">
              <tbody>
                <tr>
                  <th>調査年・公表日</th>
                  <td
                    >{String(data.metadata.year ?? "")}年（公表日: {String(
                      data.metadata.published_at ?? "",
                    )}）</td
                  >
                </tr>
                <tr>
                  <th>対象・表章項目</th>
                  <td
                    >{String(data.metadata.area ?? "全国")}、{String(
                      data.metadata.measure ?? "世帯数",
                    )}（{String(data.metadata.unit ?? "世帯")}）</td
                  >
                </tr>
                <tr>
                  <th>階級の定義</th>
                  <td
                    >{String(
                      data.metadata.asset_basis ?? "純資産総額",
                    )}。最低階級は公式の再掲内訳である「0万円未満」「0万円」「0万円超25万円未満」の3区分で表示します。元の「25万円未満」は内訳と重ねて表示せず、二重計上を防いでいます。「0万円未満」は純資産総額がマイナスの世帯です。資産額不詳の世帯は金額階級の棒には含まれません。階級の境界と開放端は統計表の公式表記を使っています。</td
                  >
                </tr>
                <tr>
                  <th>世帯数の推計</th>
                  <td
                    >世帯数は調査世帯の単純な回答数ではなく、集計用乗率を使った推計値です。</td
                  >
                </tr>
                <tr>
                  <th>値の表記</th>
                  <td
                    >「-」は該当数字なし、「0」は表章単位未満、「X」は秘匿を表します。これらを0世帯として描かず、数値が得られない階級として表示します。API応答に単位属性がないため、表題に基づき単位を「世帯」としています。世帯数は推計値で、公表値の丸めにより内訳の合計と総数が一致しない場合があります。公表値の差は補正していません。</td
                  >
                </tr>
                <tr>
                  <th>統計名</th>
                  <td>総務省統計局「令和6年全国家計構造調査」</td>
                </tr>
                <tr>
                  <th>表題</th>
                  <td>表7-131-1-1「資産額階級別世帯数」</td>
                </tr>
                <tr>
                  <th>統計表ID</th>
                  <td>
                    <div>0004056120</div>
                    <div>
                      <a
                        href="https://www.e-stat.go.jp/dbview?sid=0004056120"
                        target="_blank"
                        rel="noopener noreferrer"
                        >https://www.e-stat.go.jp/dbview?sid=0004056120</a
                      >
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </ReferencePanel>
        {/if}
        <LicenseLink />
      </aside>

      <section class="chart-panel" aria-label="純資産額階級別の世帯数">
        {#if loading}
          <div class="empty-chart"><Spinner /></div>
        {:else if error}
          <div class="empty-chart">
            <p class="error">データを読み込めませんでした。</p>
          </div>
        {:else if seriesError}
          <div class="empty-chart error" role="alert">
            <p>{seriesError}</p>
            <button onclick={retrySeries}>再試行</button>
          </div>
        {:else if currentSeries.length > 0 || seriesLoading}
          <h2 class="chart-title">純資産額階級別の世帯数</h2>
          {#if comparisonMode}<p class="hint">
              属性の比較中です。選択軸の単一区分条件は使っていません。
            </p>{/if}
          <DistributionChart
            categories={assetCategories}
            series={currentSeries}
            {orientation}
            loading={seriesLoading}
            loadingMessage={`${comparisonMode ? "比較系列" : "選択中の系列"}データを読み込んでいます…`}
            placeholderSeriesCount={Math.max(1, activeCombinations.length)}
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
        {:else}
          <div class="empty-chart">
            <h2>純資産分布を比較</h2>
            <p>世帯属性を選ぶと、ここにグラフが表示されます。</p>
            {#if !comparisonMode && data}<p class="hint" role="status">
                表示できる公表系列がありません。
              </p>{/if}
          </div>
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .series-condition-list {
    min-inline-size: 0;
    margin: 0;
    padding: var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
  }
  .series-condition-list > legend,
  .compare-options > legend {
    padding-inline: var(--space-1);
    font-weight: 600;
  }
  .intro {
    margin-top: 0;
  }
  .selectors {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-3);
    margin: var(--space-4) 0;
  }
  .selectors label,
  .comparison-controls label,
  .orientation {
    display: grid;
    align-content: start;
    gap: var(--space-2);
  }
  .orientation {
    grid-auto-flow: column;
    align-items: center;
    width: max-content;
    margin-top: var(--space-4);
  }
  select {
    min-width: 0;
  }
  .comparison-controls {
    display: grid;
    gap: var(--space-3);
    margin: var(--space-4) 0;
  }
  .compare-options {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2) var(--space-4);
    margin: var(--space-3) 0;
    padding: var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
  }
  .compare-options label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .unavailable,
  .hint {
    color: #555;
  }
  .chart-title {
    margin: 0 0 var(--space-3);
    font-size: 1.1rem;
  }
  .value-states {
    margin: var(--space-6) 0;
  }
  .value-states h2 {
    margin: 0 0 var(--space-3);
    font-size: 1.1rem;
  }
  .value-states ul {
    padding-left: 1.4rem;
  }
  .error {
    color: var(--danger-color);
  }
  @media (max-width: 50rem) {
    .compare-options {
      display: grid;
    }
  }
</style>
