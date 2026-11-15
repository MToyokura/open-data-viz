import { onMount } from "svelte";
import { AREA_LOOKUP_DATA_PATH } from "../constants/constants.js";

/** @typedef {import("../constants/types.js").AreaLookup} AreaLookup */

// Fetch area_lookup.json
/** @returns {Promise<AreaLookup>} */
async function fetchAreaLookup() {
  try {
    const response = await fetch(AREA_LOOKUP_DATA_PATH);

    if (!response.ok) {
      throw new Error(`Network response was not ok: ${response.statusText}`);
    }

    /** @type {AreaLookup} */
    const data = await response.json();
    return data;
  } catch (error) {
    console.error("Error fetching area lookup data:", error);
    throw error;
  }
}

export function useAreaLookup() {
  let areaLookup = $state(/** @type {AreaLookup | undefined} */ (undefined));
  let isLoading = $state(true);
  let error = $state(/** @type {Error | undefined} */ (undefined));

  async function load() {
    isLoading = true;
    error = undefined;
    try {
      areaLookup = await fetchAreaLookup();
    } catch (e) {
      error = e instanceof Error ? e : new Error(String(e));
    } finally {
      isLoading = false;
    }
  }

  onMount(load);

  // Return functions that return the current state values
  return {
    get areaLookup() {
      return areaLookup;
    },
    get isLoading() {
      return isLoading;
    },
    get error() {
      return error;
    },
    retry: load,
  };
}
