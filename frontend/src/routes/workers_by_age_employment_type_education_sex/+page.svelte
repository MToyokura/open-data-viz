<script>
  import { useResponsivePanels } from "#lib/useResponsivePanels.svelte.js";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
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
  let mode = $state(/** @type {"add"|"compare"} */ ("add"));
  let comparisonDimension = $state(/** @type {DimensionKey|""} */ (""));
  let educationComparisonSetId = $state("education-status");
  let comparisonCodes = $state(/** @type {string[]} */ ([]));
  let removedComparisonLabels = $state(/** @type {string[]} */ ([]));
  let comparisonSelectionNotice = $state("");
  // Conditions belonging to the mode that is currently hidden.
  let inactiveSelections = $state(
    /** @type {Record<DimensionKey, string>} */ ({
      age: "",
      employment_type: "",
      education: "",
      sex: "",
    }),
  );
  let orientation = $state(/** @type {"vertical"|"horizontal"} */ ("vertical"));
  const panels = useResponsivePanels(() => items.length);
  let serial = 0;
  /** @type {{key:DimensionKey,title:string}[]} */
  const dimensionLabels = [
    { key: "age", title: "年齢階級" },
    { key: "employment_type", title: "就業状態／雇用形態" },
    { key: "education", title: "教育" },
    { key: "sex", title: "性別" },
  ];
  const educationHierarchyNote =
    "学校区分は「在学中」「卒業」それぞれの内訳です。卒業の「大学・大学院」は、2013年から「大学」と「大学院」に分かれています。数値の四捨五入や分類不能・不詳の扱いにより、総数と内訳の合計は一致しない場合があります。";

  const selectedIndex = $derived(findIndex(selections));
  const selectedNormalItemKey = $derived(
    selectedIndex ? createItem(selectedIndex).key : "",
  );
  const selectedNormalItemAlreadyAdded = $derived(
    normalItems.some((item) => item.key === selectedNormalItemKey),
  );
  const isDataMissing = $derived(
    Boolean(options && selections.age && !selectedIndex && mode === "add"),
  );
  const items = $derived(mode === "compare" ? comparisonItems : normalItems);
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
    let choices = options.dimensions[dimension];
    if (dimension === "education") {
      const codes =
        options.education_comparison_sets.find(
          (set) => set.id === educationComparisonSetId,
        )?.category_codes ?? [];
      choices = choices.filter((choice) => codes.includes(choice.code));
    }
    return choices.map((choice) => ({
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
      inactiveSelections = { ...selections };
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
    const currentOptions = options;
    if (!currentOptions) return code;
    const name =
      currentOptions.dimensions[dimension].find((item) => item.code === code)
        ?.name ?? code;
    if (dimension !== "education") return name;
    const paths = /** @type {Record<string, string[]>} */ ({});
    /** @param {import("./constants/types.js").DisplayNode[]} nodes @param {string[]} [ancestors] */
    const visit = (nodes, ancestors = []) => {
      for (const node of nodes) {
        paths[node.code] = ancestors;
        visit(node.children ?? [], [...ancestors, node.code]);
      }
    };
    visit(currentOptions.education_display_hierarchy);
    const context = paths[code] ?? [];
    const visibleContext = context
      .filter((ancestorCode) => ancestorCode !== "00")
      .map((ancestorCode) => {
        if (ancestorCode === "01") return "在学中の内訳";
        if (ancestorCode === "05") return "卒業者の内訳";
        return (
          currentOptions.dimensions.education.find(
            (item) => item.code === ancestorCode,
          )?.name ?? ancestorCode
        ).replace(/^うち/, "");
      });
    return [...visibleContext, name].filter(Boolean).join(" > ");
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
  async function startFetch(item, comparing = mode === "compare") {
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

  function addSelectedNormalSeries() {
    if (selectedIndex && !selectedNormalItemAlreadyAdded)
      addNormalItem(selectedIndex);
  }

  /** @param {string} key */
  function removeItem(key) {
    if (mode === "compare") {
      comparisonSelectionNotice = "";
      const dimension = comparisonDimension;
      const item = comparisonItems.find((current) => current.key === key);
      if (item && dimension)
        comparisonCodes = comparisonCodes.filter(
          (code) => code !== codeForDimension(item.index, dimension),
        );
      comparisonItems = comparisonItems.filter((item) => item.key !== key);
    } else normalItems = normalItems.filter((item) => item.key !== key);
  }
  function clearItems() {
    if (mode === "compare") {
      comparisonSelectionNotice = "";
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
    if (mode === "add") {
      return;
    }
    if (!comparisonDimension) return;
    comparisonSelectionNotice = "";
    const dimension = comparisonDimension;
    if (key === dimension) return;
    const available = new Set(
      availableComparisonChoices.map((choice) => choice.code),
    );
    const removed = comparisonCodes.filter((code) => !available.has(code));
    comparisonCodes = comparisonCodes.filter((code) => available.has(code));
    removedComparisonLabels = removed.map((code) =>
      dimensionName(dimension, code),
    );
    syncComparisonItems();
  }

  /** @param {DimensionKey|""} dimension */
  function setComparison(dimension) {
    if (mode !== "compare") return;
    removedComparisonLabels = [];
    comparisonSelectionNotice = "";
    comparisonDimension = dimension;
    if (!dimension) {
      comparisonCodes = [];
      comparisonItems = [];
      return;
    }
    if (dimension === "education")
      educationComparisonSetId = "education-status";
    comparisonCodes =
      comparisonChoices
        .filter((choice) => comparisonIndex(dimension, choice.code))
        .map((choice) => choice.code) ?? [];
    comparisonSelectionNotice = `利用可能な区分を全選択しました（${comparisonCodes.length}系列）。`;
    comparisonItems = [];
    syncComparisonItems();
  }

  /** @param {string} setId */
  function setEducationComparisonSet(setId) {
    educationComparisonSetId = setId;
    comparisonCodes = comparisonChoices
      .filter((choice) => comparisonIndex("education", choice.code))
      .map((choice) => choice.code);
    comparisonSelectionNotice = `選択した区分集合から利用可能な区分を全選択しました（${comparisonCodes.length}系列）。`;
    comparisonItems = [];
    syncComparisonItems();
  }

  /** @param {"add"|"compare"} nextMode */
  function setMode(nextMode) {
    if (nextMode === mode) return;
    const previousSelections = { ...selections };
    selections = { ...inactiveSelections };
    inactiveSelections = previousSelections;
    mode = nextMode;
  }

  /** @param {string} code */
  function toggleComparisonCode(code) {
    removedComparisonLabels = [];
    comparisonSelectionNotice = "";
    comparisonCodes = comparisonCodes.includes(code)
      ? comparisonCodes.filter((selected) => selected !== code)
      : [...comparisonCodes, code];
    syncComparisonItems();
  }

  onMount(() => {
    void loadOptions();
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
            <details class="selector-panel" bind:open={panels.isSelectorOpen}>
              <summary
                >{mode === "compare"
                  ? "属性を比較"
                  : "系列を追加"}：{items.length}系列</summary
              >
              <div class="selector-panel-content">
                <div class="mode-switch" role="group" aria-label="表示モード">
                  <button
                    type="button"
                    class:active={mode === "add"}
                    aria-pressed={mode === "add"}
                    onclick={() => setMode("add")}>系列を追加</button
                  >
                  <button
                    type="button"
                    class:active={mode === "compare"}
                    aria-pressed={mode === "compare"}
                    onclick={() => setMode("compare")}>属性を比較</button
                  >
                </div>
                <p class="mode-description">
                  {#if mode === "add"}条件を選び、「系列に追加」を押すとグラフに表示されます。{:else}選んだ属性の区分を比較します。ほかの条件は全系列に共通して適用されます。各モードの条件と系列は別々に保持され、切り替えると前回の選択に戻ります。{/if}
                </p>
                {#if mode === "compare"}<section
                    class="comparison-controls"
                    aria-label="属性比較"
                  >
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
                        <option value="">属性を選択してください</option>
                        {#each dimensionLabels as dimension (dimension.key)}
                          <option value={dimension.key}
                            >{dimension.title}</option
                          >
                        {/each}
                      </select>
                    </div>
                    {#if comparisonDimension}
                      <p class="muted">
                        比較中に変更した固定条件は、このモードの選択として保持されます。
                      </p>
                      <fieldset class="compare-options">
                        <legend
                          >{dimensionLabels.find(
                            (item) => item.key === comparisonDimension,
                          )?.title}の区分</legend
                        >
                        {#if comparisonDimension === "education"}
                          <label class="comparison-set-select">
                            比較する教育区分のまとまり
                            <select
                              value={educationComparisonSetId}
                              onchange={(event) =>
                                setEducationComparisonSet(
                                  event.currentTarget.value,
                                )}
                            >
                              {#each options.education_comparison_sets as set (set.id)}
                                <option value={set.id}>{set.label}</option>
                              {/each}
                            </select>
                          </label>
                        {/if}
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
                          利用可能な区分は0件です。固定条件を変更してください。
                        </p>
                      {:else if comparisonCodes.length === 0}
                        <p class="empty" role="status">
                          比較する区分を選んでください。
                        </p>
                      {:else if comparisonCodes.length === 1}
                        <p class="empty" role="status">
                          比較には2系列以上を選択してください。
                        </p>
                      {/if}
                      {#if availableComparisonChoices.length === 1}<p
                          class="empty"
                          role="status"
                        >
                          この固定条件で利用可能な区分は1件です。比較できるデータが不足しています。
                        </p>{/if}
                      <p class="muted" role="status">
                        {comparisonSelectionNotice ||
                          `利用可能な区分：${availableComparisonChoices.length}件／選択中の比較系列：${comparisonCodes.length}件`}
                      </p>
                      {#if removedComparisonLabels.length}<p
                          class="muted"
                          role="status"
                        >
                          {removedComparisonLabels.join(
                            "、",
                          )}：共通条件の変更により選択を解除しました。
                        </p>{/if}
                    {/if}
                  </section>{/if}
                <fieldset class="series-condition-list">
                  <legend
                    >{mode === "compare"
                      ? "比較の固定条件"
                      : "系列の条件"}</legend
                  >
                  <p class="intro">
                    {mode === "compare"
                      ? "比較する属性以外の条件を、すべての比較系列に適用します。"
                      : "年齢階級・就業状態（雇用形態）・教育・性別を選び、系列を追加してください。"}
                  </p>
                  <section class="selectors" aria-label="系列の条件">
                    {#each dimensionLabels as dimension (dimension.key)}
                      <label
                        >{dimension.title}
                        <select
                          value={selections[dimension.key]}
                          disabled={mode === "compare" &&
                            comparisonDimension === dimension.key}
                          onchange={(event) =>
                            changeSelection(
                              dimension.key,
                              event.currentTarget.value,
                            )}
                        >
                          {#each options.dimensions[dimension.key] as choice (choice.code)}<option
                              value={choice.code}
                              >{dimensionName(
                                dimension.key,
                                choice.code,
                              )}</option
                            >{/each}
                        </select>
                        {#if mode === "compare" && comparisonDimension === dimension.key}<span
                            class="muted"
                            >この属性の区分を比較します（選択欄の値は比較に使いません）。</span
                          >{/if}
                      </label>
                    {/each}
                  </section>
                  {#if mode === "add"}
                    <button
                      type="button"
                      class="add-series-button"
                      disabled={!selectedIndex ||
                        selectedNormalItemAlreadyAdded}
                      onclick={addSelectedNormalSeries}
                    >
                      {selectedNormalItemAlreadyAdded
                        ? "この条件は追加済み"
                        : "系列に追加"}
                    </button>
                  {/if}
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
            <details class="added-items" bind:open={panels.isComparisonOpen}>
              <summary
                ><h2>
                  {mode === "compare" ? "比較系列" : "表示中の系列"}
                  <span>{items.length}系列</span>
                </h2></summary
              >
              <fieldset class="selected-list">
                <legend>選択中一覧</legend>
                {#if items.length === 0}
                  <p class="empty-comparison">
                    {#if mode === "compare" && !comparisonDimension}比較する属性を選んでください。{:else if mode === "compare"}比較する区分を選んでください。{:else}系列が追加されていません。{/if}
                  </p>
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
                        >
                      {/if}
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
        <ReferencePanel title="データについて">
          <h3>利用上の注意</h3>
          <ul class="reference-notes">
            <li>
              {educationHierarchyNote}
              <a
                href="https://www.stat.go.jp/data/roudou/change/2013/pdf/20121228.pdf"
                target="_blank"
                rel="noopener noreferrer"
                >2013年からの変更内容（3ページ・表7）</a
              >、<a
                href="https://www.stat.go.jp/data/roudou/rireki/nen/dt/pdf/2012.pdf"
                target="_blank"
                rel="noopener noreferrer">詳細集計の利用上の注意</a
              >
            </li>
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
          <h3>出典</h3>
          <table class="reference-table ui-table">
            <tbody>
              <tr><th>統計名</th><td>労働力調査</td></tr>
              <tr
                ><th>表題</th><td
                  >{options?.metadata.title ??
                    "年齢階級，教育，雇用形態別雇用者数"}</td
                ></tr
              >
              <tr>
                <th>統計表ID</th>
                <td>
                  <div>0003006608</div>
                  <div>
                    <a
                      href="https://www.e-stat.go.jp/dbview?sid=0003006608"
                      target="_blank"
                      rel="noopener noreferrer">e-Statで統計表を見る</a
                    >
                  </div>
                </td>
              </tr>
              <tr>
                <th>対象年／期間</th>
                <td>{options?.metadata.period ?? ""}</td>
              </tr>
              <tr>
                <th>単位</th>
                <td>{options?.metadata.unit ?? ""}</td>
              </tr>
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
        </ReferencePanel>
        <LicenseLink />
      </aside>
      <section class="chart-panel" aria-label="雇用者数の年次推移">
        {#if items.length && (mode === "add" || comparisonCodes.length > 0)}
          {#if mode === "compare" && comparisonDimension}<p
              class="muted"
              role="status"
            >
              比較属性：{dimensionLabels.find(
                (item) => item.key === comparisonDimension,
              )?.title}。固定条件：{dimensionLabels
                .filter((item) => item.key !== comparisonDimension)
                .map(
                  (item) =>
                    `${item.title} ${dimensionName(item.key, selections[item.key])}`,
                )
                .join("・")}
            </p>{/if}
          {#if mode === "compare" && comparisonCodes.length === 1}<p
              class="empty"
              role="status"
            >
              比較には2系列以上を選択してください。
            </p>{/if}
          <BarChart items={displayItems} {orientation} />
        {:else}
          <div class="empty-chart">
            <h2>雇用者数の推移を比較</h2>
            <p>
              {mode === "compare" && !comparisonDimension
                ? "比較する属性を選んでください。"
                : mode === "compare"
                  ? "比較する区分を選ぶと、ここにグラフが表示されます。"
                  : "表示する条件を選ぶと、ここにグラフが表示されます。"}
            </p>
          </div>
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .comparison-attribute {
    max-width: 24rem;
  }
  .comparison-controls p {
    margin-bottom: 0;
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
    padding: var(--space-2);
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
</style>
