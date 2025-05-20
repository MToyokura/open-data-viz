<script lang="ts">
  import type { AreaLookup, Cat02Lookup } from "./constants/types";

  let {
    autoAddEnabled = $bindable(),
    selectedCat02Code = $bindable(),
    selectedAreaCode = $bindable(),
    cat02Lookup,
    areaLookup,
    fetchAndAddData,
  }: {
    autoAddEnabled: boolean;
    selectedCat02Code: string | null;
    selectedAreaCode: string | null;
    cat02Lookup: Cat02Lookup;
    areaLookup: AreaLookup;
    fetchAndAddData: () => void;
  } = $props();
</script>

<div style="display: flex; gap: 1rem; flex-direction: column;">
  <div style="display: flex">
    <input type="checkbox" id="autoAddEnabled" bind:checked={autoAddEnabled} />
    <label for="autoAddEnabled">自動で追加する</label>
  </div>
  <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
    <select bind:value={selectedCat02Code}>
      <option value={""}>（品目を選択）</option>
      {#each cat02Lookup as cat02 (cat02.cat02_code)}
        <option value={cat02.cat02_code}>{cat02.cat02_name}</option>
      {/each}
    </select>
    <select bind:value={selectedAreaCode}>
      <option value={""}>（地域を選択）</option>
      {#each areaLookup as area (area.area_code)}
        <option value={area.area_code}>{area.area_name}</option>
      {/each}
    </select>
    {#if !autoAddEnabled}
      <button
        onclick={() => {
          fetchAndAddData();
        }}>追加</button
      >
    {/if}
  </div>
</div>

<style>
  select {
    max-width: 16em;
  }
</style>
