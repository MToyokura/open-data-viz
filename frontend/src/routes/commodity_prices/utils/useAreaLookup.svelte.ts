import { onMount } from "svelte";
import type { AreaLookup } from "../constants/types";

import { AREA_LOOKUP_DATA_PATH } from "../constants/constants";

// Fetch area_lookup.json
async function fetchAreaLookup(): Promise<AreaLookup> {
  try {
    const response = await fetch(AREA_LOOKUP_DATA_PATH);

    if (!response.ok) {
      throw new Error(`Network response was not ok: ${response.statusText}`);
    }

    const data: AreaLookup = await response.json();
    return data;
  } catch (error) {
    console.error("Error fetching area lookup data:", error);
    throw error;
  }
}

export function useAreaLookup() {
  let areaLookup: AreaLookup | undefined = $state(undefined);
  let isLoading: boolean = $state(true);
  let error: Error | undefined = $state(undefined);

  onMount(async () => {
    try {
      areaLookup = await fetchAreaLookup();
    } catch (e) {
      error = e as Error;
    } finally {
      isLoading = false;
    }
  });

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
  };
}
