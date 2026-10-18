<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import { onMount } from "svelte";
  import Spinner from "#lib/Spinner.svelte";
  import BarChart from "./BarChart.svelte";
  import { fetchOptions, fetchSeries } from "./utils/fetchData.js";

  /** @typedef {import("./constants/types.js").WorkersOptions} WorkersOptions */
  /** @typedef {"age"|"employment_type"|"education"|"sex"} DimensionKey */
  /** @typedef {{key:string,label:string,status:"loading"|"loaded"|"error", data?:import("./constants/types.js").WorkersSeries, error?:string, requestId:number, index:import("./constants/types.js").SeriesIndexItem}} Item */
  let options = $state(/** @type {WorkersOptions|null} */ (null));
  let optionsError = $state("");
  let optionsLoading = $state(true);
  let selections = $state(
    /** @type {Record<DimensionKey,string>} */ ({
      age: "",
      employment_type: "",
      education: "",
      sex: "",
    }),
  );
  let normalItems = $state(/** @type {Item[]} */ ([]));
  let comparisonItems = $state(/** @type {Item[]} */ ([]));
  let comparisonDimension = $state(/** @type {DimensionKey|""} */ (""));
  let comparisonCodes = $state(/** @type {string[]} */ ([]));
  let normalSnapshot = $state(
    /** @type {{selections:Record<DimensionKey,string>}|null} */ (null),
  );
  let orientation = $state(/** @type {"vertical"|"horizontal"} */ ("vertical"));
  let isMobile = $state(false);
  let isSelectorOpen = $state(true);
  let isComparisonOpen = $state(true);
  let serial = 0;
  /** @type {{key:DimensionKey,title:string}[]} */
  const dimensionLabels = [
    { key: "age", title: "年齢階級" },
    { key: "employment_type", title: "就業状態／雇用形態" },
    { key: "education", title: "教育" },
    { key: "sex", title: "性別" },
  ];

  const selectedIndex = $derived(findIndex(selections));
  const isDataMissing = $derived(
    Boolean(
      options && selections.age && !selectedIndex && !comparisonDimension,
    ),
  );
  const items = $derived(comparisonDimension ? comparisonItems : normalItems);
  const displayItems = $derived(
    items.map((item) => ({
      key: item.key,
      label: item.label,
      status: item.status,
      data: item.data,
    })),
  );
  const dataNotes = $derived([
    ...new Set(
      items.flatMap(
        (item) =>
          item.data?.time_value_pairs
            .map((row) => row.reason?.trim())
            .filter(
              (reason) => typeof reason === "string" && reason.length > 0,
            ) ?? [],
      ),
    ),
  ]);

  const comparisonChoices = $derived.by(() => {
    if (!comparisonDimension || !options) return [];
    const dimension = comparisonDimension;
    return options.dimensions[dimension].map((choice) => ({
      ...choice,
      index: comparisonIndex(dimension, choice.code),
    }));
  });
  const availableComparisonChoices = $derived(
    comparisonChoices.filter((choice) => choice.index),
  );

  async function loadOptions() {
    optionsLoading = true;
    optionsError = "";
    try {
      options = await fetchOptions();
      const initial = options.series[0];
      if (initial) {
        selections = selectionFromIndex(initial);
        addNormalItem(initial);
      } else {
        for (const { key } of dimensionLabels)
          selections[key] = options.dimensions[key][0]?.code ?? "";
      }
    } catch (error) {
      optionsError =
        error instanceof Error
          ? error.message
          : "選択肢の読み込みに失敗しました";
    } finally {
      optionsLoading = false;
    }
  }

  /** @param {Record<DimensionKey,string>} selected */
  function findIndex(selected) {
    return options?.series.find(
      (s) =>
        s.age_code === selected.age &&
        s.employment_type_code === selected.employment_type &&
        s.education_code === selected.education &&
        s.sex_code === selected.sex,
    );
  }

  /** @param {import("./constants/types.js").SeriesIndexItem} index */
  function selectionFromIndex(index) {
    return {
      age: index.age_code,
      employment_type: index.employment_type_code,
      education: index.education_code,
      sex: index.sex_code,
    };
  }

  /** @param {DimensionKey} dimension @param {string} code */
  function comparisonIndex(dimension, code) {
    return findIndex({ ...selections, [dimension]: code });
  }

  /** @param {import("./constants/types.js").SeriesIndexItem} index @param {DimensionKey} dimension */
  function codeForDimension(index, dimension) {
    return {
      age: index.age_code,
      employment_type: index.employment_type_code,
      education: index.education_code,
      sex: index.sex_code,
    }[dimension];
  }

  /** @param {import("./constants/types.js").SeriesIndexItem} index */
  function labelFor(index) {
    if (!options) return "";
    const educationLabel = dimensionName("education", index.education_code);
    const labels = [
      options.dimensions.age.find((x) => x.code === index.age_code)?.name,
      options.dimensions.employment_type.find(
        (x) => x.code === index.employment_type_code,
      )?.name,
      educationLabel,
      options.dimensions.sex.find((x) => x.code === index.sex_code)?.name,
    ];
    return labels.filter(Boolean).join("・");
  }

  /** @param {DimensionKey} dimension @param {string} code */
  function dimensionName(dimension, code) {
    const name =
      options?.dimensions[dimension].find((item) => item.code === code)?.name ??
      code;
    if (dimension !== "education") return name;
    return (
      {
        "03": `在学中 > ${name}`,
        "04": `在学中 > ${name}`,
        "07": `卒業 > ${name}`,
        "08": `卒業 > ${name}`,
      }[code] ?? name
    );
  }

  /** @param {boolean} comparing */
  function activeItems(comparing) {
    return comparing ? comparisonItems : normalItems;
  }

  /** @param {boolean} comparing @param {(list:Item[])=>Item[]} update */
  function updateItems(comparing, update) {
    if (comparing) comparisonItems = update(comparisonItems);
    else normalItems = update(normalItems);
  }

  /** @param {Item} item @param {boolean} comparing */
  async function startFetch(item, comparing = Boolean(comparisonDimension)) {
    const requestId = ++serial;
    const key = item.key;
    updateItems(comparing, (list) =>
      list.map((current) =>
        current.key === key
          ? {
              ...current,
              requestId,
              status: "loading",
              error: "",
              data: undefined,
            }
          : current,
      ),
    );
    try {
      const data = await fetchSeries(item.index);
      if (
        activeItems(comparing).find((x) => x.key === key)?.requestId !==
        requestId
      )
        return;
      updateItems(comparing, (list) =>
        list.map((current) =>
          current.key === key
            ? { ...current, data, status: "loaded" }
            : current,
        ),
      );
    } catch (error) {
      if (
        activeItems(comparing).find((x) => x.key === key)?.requestId !==
        requestId
      )
        return;
      const message =
        error instanceof Error ? error.message : "データの取得に失敗しました";
      updateItems(comparing, (list) =>
        list.map((current) =>
          current.key === key
            ? { ...current, error: message, status: "error" }
            : current,
        ),
      );
    }
  }

  /** @param {import("./constants/types.js").SeriesIndexItem} index @param {DimensionKey|""} compareBy */
  function createItem(index, compareBy = "") {
    const key =
      [
        index.age_code,
        index.employment_type_code,
        index.education_code,
        index.sex_code,
      ].join("|") + (compareBy ? `|${compareBy}` : "");
    return /** @type {Item} */ ({
      key,
      label: compareBy
        ? dimensionName(compareBy, index[`${compareBy}_code`])
        : labelFor(index),
      status: "loading",
      requestId: 0,
      index,
    });
  }

  /** @param {import("./constants/types.js").SeriesIndexItem} index */
  function addNormalItem(index) {
    const item = createItem(index);
    if (normalItems.some((current) => current.key === item.key)) return;
    normalItems = [...normalItems, item];
    void startFetch(item, false);
  }

  /** @param {string} key */
  function removeItem(key) {
    if (comparisonDimension) {
      const dimension = comparisonDimension;
      const item = comparisonItems.find((current) => current.key === key);
      if (item)
        comparisonCodes = comparisonCodes.filter(
          (code) => code !== codeForDimension(item.index, dimension),
        );
      comparisonItems = comparisonItems.filter((item) => item.key !== key);
    } else normalItems = normalItems.filter((item) => item.key !== key);
  }
  function clearItems() {
    if (comparisonDimension) {
      comparisonCodes = [];
      comparisonItems = [];
    } else normalItems = [];
  }

  function syncComparisonItems() {
    if (!comparisonDimension) return;
    const dimension = comparisonDimension;
    /** @type {import("./constants/types.js").SeriesIndexItem[]} */
    const indexes = comparisonCodes
      .map((code) => comparisonIndex(dimension, code))
      .filter(
        /** @param {import("./constants/types.js").SeriesIndexItem|undefined} index @returns {index is import("./constants/types.js").SeriesIndexItem} */
        (index) => Boolean(index),
      );
    const keys = new Set(
      indexes.map((index) => createItem(index, dimension).key),
    );
    comparisonItems = comparisonItems.filter((item) => keys.has(item.key));
    for (const index of indexes) {
      const item = createItem(index, dimension);
      if (comparisonItems.some((current) => current.key === item.key)) continue;
      comparisonItems = [...comparisonItems, item];
      void startFetch(item, true);
    }
  }

  /** @param {DimensionKey} key @param {string} value */
  function changeSelection(key, value) {
    selections[key] = value;
    if (!comparisonDimension) {
      if (selectedIndex) addNormalItem(selectedIndex);
      return;
    }
    if (key === comparisonDimension) return;
    const available = new Set(
      availableComparisonChoices.map((choice) => choice.code),
    );
    comparisonCodes = comparisonCodes.filter((code) => available.has(code));
    syncComparisonItems();
  }

  /** @param {DimensionKey|""} dimension */
  function setComparison(dimension) {
    if (!dimension) {
      if (normalSnapshot) selections = { ...normalSnapshot.selections };
      comparisonDimension = "";
      comparisonCodes = [];
      comparisonItems = [];
      normalSnapshot = null;
      return;
    }
    if (!comparisonDimension)
      normalSnapshot = { selections: { ...selections } };
    comparisonDimension = dimension;
    comparisonCodes =
      options?.dimensions[dimension]
        .filter((choice) => comparisonIndex(dimension, choice.code))
        .map((choice) => choice.code) ?? [];
    comparisonItems = [];
    syncComparisonItems();
  }

  /** @param {string} code */
  function toggleComparisonCode(code) {
    comparisonCodes = comparisonCodes.includes(code)
      ? comparisonCodes.filter((selected) => selected !== code)
      : [...comparisonCodes, code];
    syncComparisonItems();
  }

  onMount(() => {
    const mobileQuery = window.matchMedia("(max-width: 50rem)");
    const updateViewport = () => {
      isMobile = mobileQuery.matches;
      isSelectorOpen = !isMobile;
      isComparisonOpen = !isMobile || items.length === 0;
    };
    updateViewport();
    mobileQuery.addEventListener("change", updateViewport);
    void loadOptions();
    return () => mobileQuery.removeEventListener("change", updateViewport);
  });
</script>

<svelte:head>
  <title>年齢階級，教育，雇用形態別雇用者数 | Open Data Viz</title>
</svelte:head>

<div class="visualization-page visualization-page--workers">
  <main class="page-content">
    <WorkspaceHeader title="年齢階級，教育，雇用形態別雇用者数" />
    <div class="workspace">
      <aside class="details-column" aria-label="比較条件">
        {#if optionsLoading}<Spinner />
        {:else if optionsError}
          <div class="error" role="alert">
            <p>{optionsError}</p>
            <button onclick={loadOptions}>再試行</button>
          </div>
        {:else if options}
          <section class="control-panel" aria-label="系列の条件">
            <details class="selector-panel" bind:open={isSelectorOpen}>
              <summary>条件を選択</summary>
              <div class="selector-panel-content">
                <fieldset class="series-condition-list">
                  <legend>系列の条件</legend>
                  <p class="intro">
                    年齢階級・就業状態（雇用形態）・教育・性別を選び、年次推移を比較できます。
                  </p>
                  <section class="selectors" aria-label="系列の条件">
                    {#each dimensionLabels as dimension (dimension.key)}
                      <label
                        >{dimension.title}
                        <select
                          value={selections[dimension.key]}
                          disabled={comparisonDimension === dimension.key}
                          onchange={(event) =>
                            changeSelection(
                              dimension.key,
                              event.currentTarget.value,
                            )}
                        >
                          {#each options.dimensions[dimension.key] as choice (choice.code)}<option
                              value={choice.code}>{choice.name}</option
                            >{/each}
                        </select>
                      </label>
                    {/each}
                  </section>
                  <section class="comparison-controls" aria-label="属性比較">
                    <div class="comparison-attribute">
                      <span>属性を比較</span>
                      <select
                        aria-label="比較する属性"
                        value={comparisonDimension}
                        onchange={(event) =>
                          setComparison(
                            /** @type {DimensionKey|""} */ (
                              event.currentTarget.value
                            ),
                          )}
                      >
                        <option value="">比較しない</option>
                        {#each dimensionLabels as dimension (dimension.key)}
                          <option value={dimension.key}
                            >{dimension.title}</option
                          >
                        {/each}
                      </select>
                    </div>
                    {#if comparisonDimension}
                      <p class="muted">
                        比較中に変更した固定条件は、比較を終了すると元に戻ります。
                      </p>
                      <fieldset class="compare-options">
                        <legend
                          >{dimensionLabels.find(
                            (item) => item.key === comparisonDimension,
                          )?.title}の区分</legend
                        >
                        {#each comparisonChoices as choice (choice.code)}
                          <label class:unavailable={!choice.index}>
                            <input
                              type="checkbox"
                              checked={comparisonCodes.includes(choice.code)}
                              disabled={!choice.index}
                              title={choice.index
                                ? ""
                                : "この条件の系列はありません"}
                              onchange={() => toggleComparisonCode(choice.code)}
                            />
                            {dimensionName(comparisonDimension, choice.code)}
                            {#if !choice.index}<span class="muted"
                                >— この条件の系列はありません</span
                              >{/if}
                          </label>
                        {/each}
                      </fieldset>
                      {#if availableComparisonChoices.length === 0}
                        <p class="empty" role="status">
                          この固定条件で比較できる系列はありません。
                        </p>
                      {:else if comparisonCodes.length <= 1}
                        <p class="empty" role="status">
                          比較グラフには2区分以上が必要です。追加で選択できる区分を選んでください。
                        </p>
                      {/if}
                    {/if}
                  </section>
                  {#if isDataMissing}<p class="empty">
                      該当するデータがありません。
                    </p>{/if}
                </fieldset>
                <label class="orientation"
                  >グラフの向き
                  <select bind:value={orientation}
                    ><option value="vertical">縦棒</option><option
                      value="horizontal">横棒</option
                    ></select
                  >
                </label>
              </div>
            </details>
          </section>
          <section class="comparison-panel" aria-label="比較中のデータ">
            <details class="added-items" bind:open={isComparisonOpen}>
              <summary
                ><h2>比較する系列 <span>{items.length}系列</span></h2></summary
              >
              <fieldset class="selected-list">
                <legend>選択中一覧</legend>
                {#if items.length === 0}
                  <p class="empty-comparison">系列が追加されていません。</p>
                {/if}
                <ul>
                  {#each items as item (item.key)}<li>
                      <span>{item.label}</span>
                      {#if item.status === "loading"}<span class="muted"
                          >取得中…</span
                        >
                      {:else if item.status === "error"}<span
                          class="error"
                          role="alert"
                          >{item.error}<button onclick={() => startFetch(item)}
                            >再試行</button
                          ></span
                        >
                      {:else if item.data?.time_value_pairs.every((row) => row.value === null)}<span
                          class="muted"
                          >数値を表示できる年がありません（すべて欠損）</span
                        >{/if}
                      <button
                        class="icon-button"
                        aria-label={`${item.label}を削除`}
                        onclick={() => removeItem(item.key)}
                        ><img src={deleteIcon} alt="" /></button
                      >
                    </li>{/each}
                </ul>
              </fieldset>
              <button
                class="reset-button"
                onclick={clearItems}
                disabled={items.length === 0}>すべて消去</button
              >
            </details>
          </section>
        {/if}
        <section class="reference ui-panel ui-panel--subtle">
          <h2>データ利用上の注意</h2>
          <ul class="notes">
            <li>
              推計人口の基準切り替えに伴い、時系列に変動が生じる場合があります。また、データベース収録値は時系列接続用の補正値と異なることがあります。
            </li>
            <li>
              2011年以前の「非正規の職員・従業員」は、データベース収録値と遡及集計された結果表の値が異なる場合があります。
            </li>
            {#each dataNotes as note (note)}
              <li>{note}</li>
            {/each}
          </ul>
          <h2>出典情報</h2>
          <table class="reference-table ui-table">
            <tbody>
              <tr
                ><th>統計表ID</th><td
                  ><a
                    href="https://www.e-stat.go.jp/dbview?sid=0003006608"
                    target="_blank"
                    rel="noopener noreferrer">0003006608（e-Stat）</a
                  ></td
                ></tr
              >
              <tr><th>統計名</th><td>労働力調査</td></tr>
              <tr
                ><th>表題</th><td
                  >{options?.metadata.title ??
                    "年齢階級，教育，雇用形態別雇用者数"}</td
                ></tr
              >
              <tr
                ><th>単位・対象期間</th><td
                  >{options
                    ? `${options.metadata.unit}・${options.metadata.period}`
                    : ""}</td
                ></tr
              >
              <tr
                ><th>労働力調査について</th><td
                  ><a
                    href="https://www.stat.go.jp/data/roudou/"
                    target="_blank"
                    rel="noopener noreferrer">総務省統計局 労働力調査</a
                  ></td
                ></tr
              >
            </tbody>
          </table>
        </section>
        <LicenseLink />
      </aside>
      <section class="chart-panel" aria-label="雇用者数の年次推移">
        {#if items.length && (!comparisonDimension || comparisonCodes.length > 1)}
          <BarChart items={displayItems} {orientation} />
        {:else}
          <div class="empty-chart">
            <h2>雇用者数の推移を比較</h2>
            <p>表示する条件を選ぶと、ここにグラフが表示されます。</p>
          </div>
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .intro {
    margin-top: 0;
  }
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
  .selectors {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-3);
    margin: var(--space-4) 0;
  }
  .selectors label,
  .orientation,
  .comparison-attribute {
    display: grid;
    gap: var(--space-2);
  }
  .comparison-controls {
    margin: var(--space-4) 0;
  }
  .comparison-attribute {
    max-width: 24rem;
  }
  .comparison-controls p {
    margin-bottom: 0;
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
  .compare-options legend {
    padding: 0 var(--space-1);
  }
  .compare-options label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }
  .compare-options .unavailable {
    color: var(--text-muted-color);
  }
  select {
    min-width: 0;
    padding: var(--space-2);
  }
  .orientation {
    grid-auto-flow: column;
    align-items: center;
    width: max-content;
    margin-top: var(--space-4);
  }
  .error {
    color: var(--danger-color);
  }
  .error button {
    margin-left: var(--space-2);
  }
  .muted {
    color: var(--text-muted-color);
  }
  .empty {
    margin-top: var(--space-4);
  }
  .reference {
    margin-top: var(--space-8);
  }
  .reference h2 {
    margin: 0 0 var(--space-3);
    font-size: 1.1rem;
  }
  .reference-table a {
    word-break: break-all;
  }
  .notes {
    margin-top: 0;
  }
</style>
