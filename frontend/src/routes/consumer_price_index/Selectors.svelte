<script>
  import { buildItemDisplayHierarchy } from "./utils/itemDisplayHierarchy.js";

  /** @typedef {{ code: string, name: string, level?: string, parentCode?: string }} Item */
  /** @typedef {{ code: string, name: string }} Region */
  /** @typedef {{ itemCode: string, areaCode: string }} SelectedCombination */
  /** @type {{ items: Item[], regions: Region[], selectedItemCodes: string[], selectedAreaCodes: string[], excludedCombinationKeys: string[], selectedCombinations: SelectedCombination[] }} */
  let {
    items,
    regions,
    selectedItemCodes = $bindable(),
    selectedAreaCodes = $bindable(),
    excludedCombinationKeys = $bindable(),
    selectedCombinations,
  } = $props();
  let itemQuery = $state("");
  let regionQuery = $state("");
  let displayItems = $derived(buildItemDisplayHierarchy(items));
  let filteredItems = $derived(
    displayItems.filter((item) => item.name.includes(itemQuery.trim())),
  );
  let filteredRegions = $derived(
    regions.filter((region) => region.name.includes(regionQuery.trim())),
  );

  /** @param {string} itemCode */
  function itemIsSelected(itemCode) {
    return (
      selectedAreaCodes.length > 0 &&
      selectedCombinations.filter((entry) => entry.itemCode === itemCode)
        .length === selectedAreaCodes.length
    );
  }

  /** @param {string} itemCode */
  function itemIsPartial(itemCode) {
    const count = selectedCombinations.filter(
      (entry) => entry.itemCode === itemCode,
    ).length;
    return count > 0 && count < selectedAreaCodes.length;
  }

  /** @param {string} areaCode */
  function areaIsSelected(areaCode) {
    return (
      selectedItemCodes.length > 0 &&
      selectedCombinations.filter((entry) => entry.areaCode === areaCode)
        .length === selectedItemCodes.length
    );
  }

  /** @param {string} areaCode */
  function areaIsPartial(areaCode) {
    const count = selectedCombinations.filter(
      (entry) => entry.areaCode === areaCode,
    ).length;
    return count > 0 && count < selectedItemCodes.length;
  }

  /** @param {string[]} values @param {string} value @param {boolean} checked */
  function toggle(values, value, checked) {
    return checked
      ? [...new Set([...values, value])]
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
        <label class="option" style:--depth={item.displayDepth}>
          <input
            type="checkbox"
            checked={itemIsSelected(item.code)}
            onchange={(event) => {
              selectedItemCodes = toggle(
                selectedItemCodes,
                item.code,
                event.currentTarget.checked,
              );
              if (event.currentTarget.checked) {
                excludedCombinationKeys = excludedCombinationKeys.filter(
                  (key) => !key.startsWith(`${item.code}:`),
                );
              }
            }}
            indeterminate={itemIsPartial(item.code)}
            aria-checked={itemIsPartial(item.code)
              ? "mixed"
              : itemIsSelected(item.code)}
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
            checked={areaIsSelected(region.code)}
            onchange={(event) => {
              selectedAreaCodes = toggle(
                selectedAreaCodes,
                region.code,
                event.currentTarget.checked,
              );
              if (event.currentTarget.checked) {
                excludedCombinationKeys = excludedCombinationKeys.filter(
                  (key) => !key.endsWith(`:${region.code}`),
                );
              }
            }}
            indeterminate={areaIsPartial(region.code)}
            aria-checked={areaIsPartial(region.code)
              ? "mixed"
              : areaIsSelected(region.code)}
          />
          {region.name}
        </label>
      {:else}
        <p class="muted">該当する地域がありません。</p>
      {/each}
    </div>
  </fieldset>
</div>

<style>
  .selector-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  fieldset {
    min-width: 0;
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
  }
  legend {
    padding: 0 var(--space-1);
    font-weight: 600;
  }
  .search-label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-bottom: var(--space-2);
    font-size: 0.875rem;
  }
  .search-label input {
    width: 100%;
  }
  .options {
    display: flex;
    flex-direction: column;
    max-height: 12rem;
    overflow-y: auto;
  }
  .option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) 0;
    cursor: pointer;
  }
  .option input[type="checkbox"] {
    width: 13px;
    height: 13px;
    flex: 0 0 13px;
  }
  .option input:indeterminate {
    appearance: none;
    border: 1px solid var(--focus-color);
    border-radius: 0.2rem;
    background-color: var(--focus-color);
    background-image: linear-gradient(
      var(--surface-color),
      var(--surface-color)
    );
    background-repeat: no-repeat;
    background-position: center;
    background-size: 60% 3px;
  }
  .item-options .option {
    padding-left: calc(var(--depth) * 0.65rem);
  }
  .muted {
    color: var(--text-muted-color);
    font-size: 0.875rem;
  }
</style>
