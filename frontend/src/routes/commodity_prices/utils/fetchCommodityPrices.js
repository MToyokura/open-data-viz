import { COMMODITY_PRICES_TRANSFORMED_DATA_PATH } from "../constants/constants.js";

/** @typedef {import("../constants/types.js").CommodityPrices} CommodityPrices */

// Fetch data from the API
/**
 * @param {{ cat02Code: string, areaCode: string }} params
 * @returns {Promise<CommodityPrices | undefined>}
 */
export async function fetchCommodityPrices({ cat02Code, areaCode }) {
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

    /** @type {CommodityPrices} */
    const data = await response.json();
    return data;
  } catch (error) {
    console.error("Error fetching data:", error);
    return undefined;
  }
}
