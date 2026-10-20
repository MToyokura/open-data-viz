import { onMount } from "svelte";

/** @param {() => number} getSeriesCount */
export function useResponsivePanels(getSeriesCount) {
  const panels = $state({ isSelectorOpen: true, isComparisonOpen: true });

  onMount(() => {
    const query = window.matchMedia("(max-width: 50rem)");
    const updateViewport = () => {
      panels.isSelectorOpen = !query.matches;
      panels.isComparisonOpen = !query.matches || getSeriesCount() === 0;
    };
    updateViewport();
    query.addEventListener("change", updateViewport);
    return () => query.removeEventListener("change", updateViewport);
  });

  return panels;
}
