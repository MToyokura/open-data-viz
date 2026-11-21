<script>
  /** @typedef {import("./constants/types.js").AreaLookup} AreaLookup */
  /** @typedef {import("./constants/types.js").Cat02Lookup} Cat02Lookup */
  /** @typedef {{ cat02Code: string, areaCode: string }} Combination */
  /** @type {{ cat02Lookup: Cat02Lookup, areaLookup: AreaLookup, selectedCat02Codes: string[], selectedAreaCodes: string[], excludedPairKeys: string[], selectedCombinations: Combination[] }} */
  let { cat02Lookup, areaLookup, selectedCat02Codes = $bindable(), selectedAreaCodes = $bindable(), excludedPairKeys = $bindable(), selectedCombinations } = $props();
  let itemQuery = $state("");
  let areaQuery = $state("");
  let filteredItems = $derived(cat02Lookup.filter((item) => item.cat02_name.includes(itemQuery.trim())));
  let filteredAreas = $derived(areaLookup.filter((item) => item.area_name.includes(areaQuery.trim())));
  /** @param {string[]} values @param {string} value @param {boolean} checked */
  const toggle = (values, value, checked) => checked ? [...new Set([...values, value])] : values.filter((entry) => entry !== value);
  /** @param {string} code */
  function itemIsSelected(code) { return selectedAreaCodes.length > 0 && selectedCombinations.filter((entry) => entry.cat02Code === code).length === selectedAreaCodes.length; }
  /** @param {string} code */
  function itemIsPartial(code) { const n = selectedCombinations.filter((entry) => entry.cat02Code === code).length; return n > 0 && n < selectedAreaCodes.length; }
  /** @param {string} code */
  function areaIsSelected(code) { return selectedCat02Codes.length > 0 && selectedCombinations.filter((entry) => entry.areaCode === code).length === selectedCat02Codes.length; }
  /** @param {string} code */
  function areaIsPartial(code) { const n = selectedCombinations.filter((entry) => entry.areaCode === code).length; return n > 0 && n < selectedCat02Codes.length; }
</script>

<div class="selector-group">
  <fieldset>
    <legend>品目（複数選択可）</legend>
    <label class="search-label"><span>品目を検索</span><input type="search" bind:value={itemQuery} placeholder="例：食パン" /></label>
    <div class="options">
      {#each filteredItems as item (item.cat02_code)}
        <label class="option">
          <input type="checkbox" checked={itemIsSelected(item.cat02_code)} indeterminate={itemIsPartial(item.cat02_code)} aria-checked={itemIsPartial(item.cat02_code) ? "mixed" : itemIsSelected(item.cat02_code)} onchange={(event) => { selectedCat02Codes = toggle(selectedCat02Codes, item.cat02_code, event.currentTarget.checked); if (event.currentTarget.checked) excludedPairKeys = excludedPairKeys.filter((key) => !key.startsWith(`${item.cat02_code}:`)); }} />
          {item.cat02_name}
        </label>
      {:else}<p class="muted">該当する品目がありません。</p>{/each}
    </div>
  </fieldset>
  <fieldset>
    <legend>地域（複数選択可）</legend>
    <label class="search-label"><span>地域を検索</span><input type="search" bind:value={areaQuery} placeholder="例：札幌市" /></label>
    <div class="options">
      {#each filteredAreas as area (area.area_code)}
        <label class="option">
          <input type="checkbox" checked={areaIsSelected(area.area_code)} indeterminate={areaIsPartial(area.area_code)} aria-checked={areaIsPartial(area.area_code) ? "mixed" : areaIsSelected(area.area_code)} onchange={(event) => { selectedAreaCodes = toggle(selectedAreaCodes, area.area_code, event.currentTarget.checked); if (event.currentTarget.checked) excludedPairKeys = excludedPairKeys.filter((key) => !key.endsWith(`:${area.area_code}`)); }} />
          {area.area_name}
        </label>
      {:else}<p class="muted">該当する地域がありません。</p>{/each}
    </div>
  </fieldset>
</div>
<style>
  .selector-group { display:flex; flex-direction:column; gap:var(--space-4); }
  fieldset { min-width:0; margin:0; padding:var(--space-2) var(--space-3); border:var(--border-default); border-radius:var(--radius-md); }
  legend { padding:0 var(--space-1); font-weight:600; }
  .search-label { display:flex; flex-direction:column; gap:var(--space-1); margin-bottom:var(--space-2); font-size:.875rem; }
  .search-label input { width:100%; }
  .options { display:flex; flex-direction:column; max-height:12rem; overflow-y:auto; }
  .option { display:flex; align-items:center; gap:var(--space-2); padding:var(--space-1) 0; cursor:pointer; }
  .option input[type="checkbox"] { width:13px; height:13px; flex:0 0 13px; }
  .option input:indeterminate { appearance:none; border:1px solid var(--focus-color); border-radius:.2rem; background-color:var(--focus-color); background-image:linear-gradient(var(--surface-color),var(--surface-color)); background-repeat:no-repeat; background-position:center; background-size:60% 3px; }
  .muted { color:var(--text-muted-color); font-size:.875rem; }
</style>
