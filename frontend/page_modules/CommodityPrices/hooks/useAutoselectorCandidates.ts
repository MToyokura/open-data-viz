import useSWR from "swr";
import { staticAssetsBaseUrl } from "../../../pages/_app";
import { fetcher } from "../../fetcher";
import { AutoselectorCandidates } from "../types/AutoselectorCandidates";

export function useCommodityCandidates() {
  const {
    data: responseCommodityCandidates,
    isLoading,
    error,
  } = useSWR<AutoselectorCandidates>(
    `${staticAssetsBaseUrl}/commodity_prices/autoselect_data/commodity_selections.json`,
    fetcher
  );
  return {
    responseCommodityCandidates: responseCommodityCandidates,
    isLoading: isLoading,
    error: error,
  };
}

export function useRegionCandidates() {
  const {
    data: responseRegionCandidates,
    isLoading,
    error,
  } = useSWR<AutoselectorCandidates>(
    `${staticAssetsBaseUrl}/commodity_prices/autoselect_data/region_selections.json`,
    fetcher
  );
  return {
    responseRegionCandidates: responseRegionCandidates,
    isLoading: isLoading,
    error: error,
  };
}
