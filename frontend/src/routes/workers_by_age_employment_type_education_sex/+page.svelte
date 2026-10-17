<script>
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import { onMount } from "svelte";
  import Spinner from "#lib/Spinner.svelte";
  import { VITE_ROOT_PATH } from "#lib/paths.js";
  import BarChart from "./BarChart.svelte";
  import { fetchOptions, fetchSeries } from "./utils/fetchData.js";

  /** @typedef {import("./constants/types.js").WorkersOptions} WorkersOptions */
  /** @typedef {"age"|"employment_type"|"education"|"sex"} DimensionKey */
  /** @typedef {{key:string,label:string,status:"loading"|"loaded"|"error", data?:import("./constants/types.js").WorkersSeries, error?:string, requestId:number, index:import("./constants/types.js").SeriesIndexItem}} Item */
  let options = $state(/** @type {WorkersOptions|null} */ (null));
  let optionsError = $state("");
  let optionsLoading = $state(true);
  let selections = $state(/** @type {Record<DimensionKey,string>} */ ({ age: "", employment_type: "", education: "", sex: "" }));
  let items = $state(/** @type {Item[]} */ ([]));
  let orientation = $state(/** @type {"vertical"|"horizontal"} */ ("vertical"));
  let serial = 0;
  /** @type {{key:DimensionKey,title:string}[]} */
  const dimensionLabels = [
    { key: "age", title: "年齢階級" },
    { key: "employment_type", title: "就業状態／雇用形態" },
    { key: "education", title: "教育" },
    { key: "sex", title: "性別" },
  ];

  const condition = $derived({ age_code: selections.age, employment_type_code: selections.employment_type, education_code: selections.education, sex_code: selections.sex });
  const selectedIndex = $derived(options?.series.find((s) => s.age_code === condition.age_code && s.employment_type_code === condition.employment_type_code && s.education_code === condition.education_code && s.sex_code === condition.sex_code));
  const isDataMissing = $derived(Boolean(options && selections.age && !selectedIndex));
  const displayItems = $derived(items.map((item) => ({ key: item.key, label: item.label, status: item.status, data: item.data })));

  async function loadOptions() {
    optionsLoading = true;
    optionsError = "";
    try {
      options = await fetchOptions();
      for (const { key } of dimensionLabels) selections[key] = options.dimensions[key][0]?.code ?? "";
    } catch (error) {
      optionsError = error instanceof Error ? error.message : "選択肢の読み込みに失敗しました";
    } finally { optionsLoading = false; }
  }

  /** @param {import("./constants/types.js").SeriesIndexItem} index */
  function labelFor(index) {
    if (!options) return "";
    const educationName = options.dimensions.education.find((x) => x.code === index.education_code)?.name;
    const educationLabel = ({
      "03": `在学中 > ${educationName}`,
      "04": `在学中 > ${educationName}`,
      "07": `卒業 > ${educationName}`,
      "08": `卒業 > ${educationName}`,
    })[index.education_code] ?? educationName;
    const labels = [
      options.dimensions.age.find((x) => x.code === index.age_code)?.name,
      options.dimensions.employment_type.find((x) => x.code === index.employment_type_code)?.name,
      educationLabel,
      options.dimensions.sex.find((x) => x.code === index.sex_code)?.name,
    ];
    return labels.filter(Boolean).join("・");
  }

  /** @param {Item} item */
  async function startFetch(item) {
    const requestId = ++serial;
    const key = item.key;
    items = items.map((current) => current.key === key
      ? { ...current, requestId, status: "loading", error: "", data: undefined }
      : current);
    try {
      const data = await fetchSeries(item.index);
      if (items.find((x) => x.key === key)?.requestId !== requestId) return;
      items = items.map((current) => current.key === key ? { ...current, data, status: "loaded" } : current);
    } catch (error) {
      if (items.find((x) => x.key === key)?.requestId !== requestId) return;
      const message = error instanceof Error ? error.message : "データの取得に失敗しました";
      items = items.map((current) => current.key === key ? { ...current, error: message, status: "error" } : current);
    }
  }

  function addSeries() {
    if (!selectedIndex || !options || optionsLoading || optionsError) return;
    const key = [condition.age_code, condition.employment_type_code, condition.education_code, condition.sex_code].join("|");
    if (items.some((item) => item.key === key)) return;
    const item = /** @type {Item} */ ({ key, label: labelFor(selectedIndex), status: "loading", requestId: 0, index: selectedIndex });
    items = [...items, item];
    void startFetch(item);
  }

  /** @param {string} key */
  function removeItem(key) { items = items.filter((item) => item.key !== key); }
  function clearItems() { items = []; serial += 1; }

  onMount(() => { void loadOptions(); });
</script>

<svelte:head><title>年齢階級，教育，雇用形態別雇用者数 | Open Data Viz</title></svelte:head>

<div class="page-shell"><main class="page-content">
  <h1>年齢階級，教育，雇用形態別雇用者数</h1>
  <p>年齢階級・就業状態（雇用形態）・教育・性別を選び、年次推移を比較できます。</p>

  {#if optionsLoading}<Spinner />
  {:else if optionsError}
    <div class="error" role="alert"><p>{optionsError}</p><button onclick={loadOptions}>再試行</button></div>
  {:else if options}
    <section class="selectors" aria-label="系列の条件">
      {#each dimensionLabels as dimension (dimension.key)}
        <label>{dimension.title}
          <select bind:value={selections[dimension.key]}>
            {#each options.dimensions[dimension.key] as choice (choice.code)}<option value={choice.code}>{choice.name}</option>{/each}
          </select>
        </label>
      {/each}
    </section>
    <div class="actions"><button onclick={addSeries} disabled={!selectedIndex}>系列を追加</button>
      <label class="orientation">グラフの向き
        <select bind:value={orientation}><option value="vertical">縦棒</option><option value="horizontal">横棒</option></select>
      </label>
      <button class="secondary" onclick={clearItems} disabled={items.length === 0}>すべて消去</button>
    </div>
    {#if isDataMissing}<p class="empty">該当するデータがありません。</p>{/if}
    {#if items.length}
      <section class="added-items" aria-label="追加した系列"><h2>比較する系列</h2><ul>
        {#each items as item (item.key)}<li>
          <span>{item.label}</span>
          {#if item.status === "loading"}<span class="muted">取得中…</span>
          {:else if item.status === "error"}<span class="error" role="alert">{item.error}<button onclick={() => startFetch(item)}>再試行</button></span>
          {:else if item.data?.time_value_pairs.every((row) => row.value === null)}<span class="muted">数値を表示できる年がありません（すべて欠損）</span>{/if}
          <button class="delete-button" aria-label={`${item.label}を削除`} onclick={() => removeItem(item.key)}><img src={deleteIcon} alt="" /></button>
        </li>{/each}
      </ul></section>
      <BarChart items={displayItems} {orientation} />
    {/if}
  {/if}

  <section class="reference ui-panel ui-panel--subtle">
    <h2>データ利用上の注意</h2>
    <ul class="notes">
      <li>推計人口の基準切り替えに伴い、時系列に変動が生じる場合があります。また、データベース収録値は時系列接続用の補正値と異なることがあります。</li>
      <li>2011年以前の「非正規の職員・従業員」は、データベース収録値と遡及集計された結果表の値が異なる場合があります。</li>
    </ul>
    <h2>出典情報</h2><table class="reference-table ui-table"><tbody>
      <tr><th>統計表ID</th><td><a href="https://www.e-stat.go.jp/dbview?sid=0003006608" target="_blank" rel="noopener noreferrer">0003006608（e-Stat）</a></td></tr>
      <tr><th>統計名</th><td>労働力調査</td></tr>
      <tr><th>表題</th><td>{options?.metadata.title ?? "年齢階級，教育，雇用形態別雇用者数"}</td></tr>
      <tr><th>単位・対象期間</th><td>{options ? `${options.metadata.unit}・${options.metadata.period}` : ""}</td></tr>
      <tr><th>労働力調査について</th><td><a href="https://www.stat.go.jp/data/roudou/" target="_blank" rel="noopener noreferrer">総務省統計局 労働力調査</a></td></tr>
    </tbody></table>
  </section>
  <p><a href={`${VITE_ROOT_PATH}/`}>トップページへ戻る</a></p>
</main></div>

<style>
  .selectors { display: grid; grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr)); gap: var(--space-4); margin: var(--space-6) 0; }
  .selectors label, .orientation { display: grid; gap: var(--space-2); }
  select { min-width: 0; padding: var(--space-2); }
  .actions { display: flex; align-items: center; flex-wrap: wrap; gap: var(--space-3); }
  .orientation { grid-auto-flow: column; align-items: center; }
  .secondary { margin-left: auto; }
  .added-items { max-width: 55rem; margin-top: var(--space-6); }
  .added-items h2, .reference h2 { margin: 0 0 var(--space-3); font-size: 1.1rem; }
  .added-items ul { display: flex; flex-direction: column; max-height: 16rem; overflow-y: auto; margin: 0; padding: 0; border: var(--border-default); border-radius: var(--radius-md); list-style: none; }
  .added-items li { display: flex; align-items: center; flex-wrap: wrap; gap: var(--space-3); padding: var(--space-2) var(--space-3); border-bottom: var(--border-default); }
  .added-items li:last-child { border-bottom: 0; }
  .added-items li > span:first-child { flex: 1 1 12rem; }
  .delete-button { display: inline-flex; align-items: center; padding: var(--space-1); border: 0; border-radius: var(--radius-round); background: transparent; cursor: pointer; }
  .delete-button:hover { background: var(--surface-hover-color); }
  .delete-button img { width: 24px; height: 24px; }
  .error { color: var(--danger-color); }
  .error button { margin-left: var(--space-2); }
  .muted { color: var(--text-muted-color); }
  .empty { margin-top: var(--space-4); }
  .reference { margin-top: var(--space-8); }
  .notes { margin-top: 0; }
  .reference-table a { word-break: break-all; }
  @media (max-width: 40rem) { .secondary { margin-left: 0; } }
</style>
