import { onMount } from "svelte";

/**
 * @param {() => number} getSeriesCount
 * @param {{ mobileOpen?: boolean }} [options]
 */
export function useResponsivePanels(
  getSeriesCount,
  { mobileOpen = false } = {},
) {
  const panels = $state({ isSelectorOpen: true, isComparisonOpen: true });

  onMount(() => {
    // Keep this in sync with the shared workspace column switch.
    const query = window.matchMedia("(max-width: 50rem)");
    const updateViewport = () => {
      panels.isSelectorOpen = !query.matches || mobileOpen;
      panels.isComparisonOpen =
        !query.matches || mobileOpen || getSeriesCount() === 0;
    };
    updateViewport();
    query.addEventListener("change", updateViewport);
    return () => query.removeEventListener("change", updateViewport);
  });

  return panels;
}
