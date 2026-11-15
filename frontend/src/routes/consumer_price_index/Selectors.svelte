<script>
  /** @typedef {{ code: string, name: string, level?: string, parentCode?: string }} Item */
  /** @typedef {{ code: string, name: string }} Region */
  /** @type {{ items: Item[], regions: Region[], selectedItemCodes: string[], selectedAreaCodes: string[] }} */
  let {
    items,
    regions,
    selectedItemCodes = $bindable(),
    selectedAreaCodes = $bindable(),
  } = $props();
  let itemQuery = $state("");
  let regionQuery = $state("");
  let filteredItems = $derived(
    items.filter((item) => item.name.includes(itemQuery.trim())),
  );
  let filteredRegions = $derived(
    regions.filter((region) => region.name.includes(regionQuery.trim())),
  );

  function toggle(values, value, checked) {
    return checked
      ? [...values, value]
      : values.filter((entry) => entry !== value);
  }
</script>

<div class="selector-group">
  <fieldset>
    <legend>品目（複数選択可）</legend>
    <label class="search-label">
      <span>品目を検索</span>
      <input type="search" bind:value={itemQuery} placeholder="例：食料" />
    </label>
    <div class="options item-options">
      {#each filteredItems as item (item.code)}
        <label class="option" style:--depth={Math.max(0, Number(item.level ?? 1) - 1)}>
          <input
            type="checkbox"
            disabled={!selectedItemCodes.includes(item.code) && selectedItemCodes.length * Math.max(1, selectedAreaCodes.length) >= 40}
            checked={selectedItemCodes.includes(item.code)}
            onchange={(event) => {
              selectedItemCodes = toggle(
                selectedItemCodes,
                item.code,
                event.currentTarget.checked,
              );
            }}
          />
          {item.name}
        </label>
      {:else}
        <p class="muted">該当する品目がありません。</p>
      {/each}
    </div>
  </fieldset>

  <fieldset>
    <legend>地域（複数選択可）</legend>
    <label class="search-label">
      <span>地域を検索</span>
      <input type="search" bind:value={regionQuery} placeholder="例：全国" />
    </label>
    <div class="options">
      {#each filteredRegions as region (region.code)}
        <label class="option">
          <input
            type="checkbox"
            disabled={!selectedAreaCodes.includes(region.code) && selectedAreaCodes.length * Math.max(1, selectedItemCodes.length) >= 40}
            checked={selectedAreaCodes.includes(region.code)}
            onchange={(event) => {
              selectedAreaCodes = toggle(
                selectedAreaCodes,
                region.code,
                event.currentTarget.checked,
              );
            }}
          />
          {region.name}
        </label>
      {:else}
        <p class="muted">該当する地域がありません。</p>
      {/each}
    </div>
  </fieldset>
</div>

<p class="selection-limit">一度に比較できる系列は40件までです。系列数は選択した品目数 × 地域数です。</p>

<style>
  .selector-group { display: flex; flex-direction: column; gap: var(--space-4); }
  fieldset { min-width: 0; margin: 0; padding: var(--space-2) var(--space-3); border: var(--border-default); border-radius: var(--radius-md); }
  legend { padding: 0 var(--space-1); font-weight: 600; }
  .search-label { display: flex; flex-direction: column; gap: var(--space-1); margin-bottom: var(--space-2); font-size: 0.875rem; }
  .search-label input { width: 100%; }
  .options { display: flex; flex-direction: column; max-height: 12rem; overflow-y: auto; }
  .option { display: flex; align-items: center; gap: var(--space-2); padding: var(--space-1) 0; cursor: pointer; }
  .item-options .option { padding-left: calc(var(--depth) * 0.65rem); }
  .muted { color: var(--text-muted-color); font-size: 0.875rem; }
  .selection-limit { margin: 0; color: var(--text-muted-color); font-size: 0.8rem; }
</style>
