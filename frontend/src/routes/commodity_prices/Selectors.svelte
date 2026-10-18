<script>
  /** @typedef {import("./constants/types.js").AreaLookup} AreaLookup */
  /** @typedef {import("./constants/types.js").Cat02Lookup} Cat02Lookup */

  /** @type {{
   * selectedCat02Code: string,
   * selectedAreaCodesByCat02: Record<string, string[]>,
   * cat02Lookup: Cat02Lookup,
   * areaLookup: AreaLookup,
   * }}
   */
  let {
    selectedCat02Code = $bindable(),
    selectedAreaCodesByCat02 = $bindable(),
    cat02Lookup,
    areaLookup,
  } = $props();
  let itemQuery = $state("");
  let filteredCat02Lookup = $derived(
    cat02Lookup.filter((item) => item.cat02_name.includes(itemQuery.trim())),
  );
  /** @param {string} areaCode @param {boolean} checked */
  function updateAreaSelection(areaCode, checked) {
    if (!selectedCat02Code) return;

    const selectedAreaCodes = selectedAreaCodesByCat02[selectedCat02Code] ?? [];
    selectedAreaCodesByCat02 = {
      ...selectedAreaCodesByCat02,
      [selectedCat02Code]: checked
        ? [...selectedAreaCodes, areaCode]
        : selectedAreaCodes.filter((code) => code !== areaCode),
    };
  }
</script>

<div class="selector-group">
  <div class="selector-controls">
    <fieldset class="item-selector">
      <legend>品目</legend>
      <label class="search-label">
        <span>品目を検索</span>
        <input type="search" bind:value={itemQuery} placeholder="例：食パン" />
      </label>
      <div class="item-options">
        {#each filteredCat02Lookup as cat02 (cat02.cat02_code)}
          <label class="item-option">
            <input
              type="radio"
              value={cat02.cat02_code}
              bind:group={selectedCat02Code}
            />
            {cat02.cat02_name}
          </label>
        {:else}
          <p class="no-results">該当する品目がありません。</p>
        {/each}
      </div>
    </fieldset>
    <fieldset class="area-selector">
      <legend>地域（複数選択可）</legend>
      <div class="area-options">
        {#each areaLookup as area (area.area_code)}
          <label class="area-option">
            <input
              type="checkbox"
              disabled={!selectedCat02Code}
              checked={selectedAreaCodesByCat02[selectedCat02Code]?.includes(
                area.area_code,
              ) ?? false}
              onchange={(event) =>
                updateAreaSelection(
                  area.area_code,
                  event.currentTarget.checked,
                )}
            />
            {area.area_name}
          </label>
        {/each}
      </div>
    </fieldset>
  </div>
</div>

<style>
  .selector-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .selector-controls {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .area-selector {
    min-width: 0;
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
  }

  .item-selector {
    min-width: 0;
    margin: 0;
    padding: var(--space-2) var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
  }

  .item-selector legend {
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

  .no-results {
    margin: var(--space-2) 0;
    color: #555;
    font-size: 0.875rem;
  }

  .item-options {
    display: flex;
    flex-direction: column;
    max-height: 12rem;
    overflow-y: auto;
  }

  .item-option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) 0;
    cursor: pointer;
  }

  .area-selector legend {
    padding: 0 var(--space-1);
    font-weight: 600;
  }

  .area-options {
    display: flex;
    flex-direction: column;
    max-height: 12rem;
    overflow-y: auto;
  }

  .area-option {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) 0;
    cursor: pointer;
  }
</style>
