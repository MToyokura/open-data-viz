import { onMount } from "svelte";
import { CAT02_LOOKUP_DATA_PATH } from "../constants/constants";
import type { Cat02Lookup } from "../constants/types";

// Fetch cat02_lookup.json
export async function fetchCat02Lookup(): Promise<Cat02Lookup> {
  try {
    const response = await fetch(CAT02_LOOKUP_DATA_PATH);

    if (!response.ok) {
      throw new Error(`Network response was not ok: ${response.statusText}`);
    }

    const data: Cat02Lookup = await response.json();
    return data;
  } catch (error) {
    console.error("Error fetching cat02 lookup data:", error);
    throw error;
  }
}

export function useCat02Lookup() {
  let cat02Lookup: Cat02Lookup | undefined = $state(undefined);
  let isLoading: boolean = $state(true);
  let error: Error | undefined = $state(undefined);

  onMount(async () => {
    try {
      cat02Lookup = await fetchCat02Lookup();
    } catch (e) {
      error = e as Error;
    } finally {
      isLoading = false;
    }
  });

  // Return functions that return the current state values
  return {
    get cat02Lookup() {
      return cat02Lookup;
    },
    get isLoading() {
      return isLoading;
    },
    get error() {
      return error;
    },
  };
}
