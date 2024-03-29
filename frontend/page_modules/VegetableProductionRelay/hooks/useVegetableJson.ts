import useSWRImmutable from "swr/immutable";
import { staticAssetsBaseUrl } from "../../../pages/_app";
import { fetcher } from "../../fetcher";
import { VegetableJson } from "../types/AmountPerPrefectureOfMonth";

export function useVegetableJson(vegetable: string) {
  const {
    data: responseVegetableJson,
    isLoading,
    error,
  } = useSWRImmutable<VegetableJson>(
    `${staticAssetsBaseUrl}/monthly_vegetable_market_amount_by_prefecture/${vegetable}.json`,
    fetcher
  );
  return {
    responseVegetableJson: responseVegetableJson,
    isLoading: isLoading,
    error: error,
  };
}
