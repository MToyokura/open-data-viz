import { COMMODITY_PRICES_TRANSFORMED_DATA_PATH } from "../constants/constants";
import type { CommodityPrices } from "../constants/types";

// Fetch data from the API
export async function fetchCommodityPrices({
  cat02Code,
  areaCode,
}: {
  cat02Code: string;
  areaCode: string;
}): Promise<CommodityPrices | undefined> {
  try {
    const response = await fetch(
      `${COMMODITY_PRICES_TRANSFORMED_DATA_PATH}/commodity_prices_${cat02Code}_${areaCode}.json`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      },
    );

    if (!response.ok) {
      throw new Error("Network response was not ok");
    }

    const data: CommodityPrices = await response.json();
    return data;
  } catch (error) {
    console.error("Error fetching data:", error);
    return undefined;
  }
}
