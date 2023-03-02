import {
  AmountPerPrefectureOfMonth,
  VegetableJson,
} from "../types/AmountPerPrefectureOfMonth";

export function createMonthlyData(
  vegetableJson: VegetableJson
): AmountPerPrefectureOfMonth[] {
  return [
    // 1桁の月が全角になっているのはもとのデータでそうなっているのに合わせているから
    vegetableJson[2018]["１月"],
    vegetableJson[2018]["２月"],
    vegetableJson[2018]["３月"],
    vegetableJson[2018]["４月"],
    vegetableJson[2018]["５月"],
    vegetableJson[2018]["６月"],
    vegetableJson[2018]["７月"],
    vegetableJson[2018]["８月"],
    vegetableJson[2018]["９月"],
    vegetableJson[2018]["10月"],
    vegetableJson[2018]["11月"],
    vegetableJson[2018]["12月"],
    vegetableJson[2019]["１月"],
    vegetableJson[2019]["２月"],
    vegetableJson[2019]["３月"],
    vegetableJson[2019]["４月"],
    vegetableJson[2019]["５月"],
    vegetableJson[2019]["６月"],
    vegetableJson[2019]["７月"],
    vegetableJson[2019]["８月"],
    vegetableJson[2019]["９月"],
    vegetableJson[2019]["10月"],
    vegetableJson[2019]["11月"],
    vegetableJson[2019]["12月"],
    vegetableJson[2020]["１月"],
    vegetableJson[2020]["２月"],
    vegetableJson[2020]["３月"],
    vegetableJson[2020]["４月"],
    vegetableJson[2020]["５月"],
    vegetableJson[2020]["６月"],
    vegetableJson[2020]["７月"],
    vegetableJson[2020]["８月"],
    vegetableJson[2020]["９月"],
    vegetableJson[2020]["10月"],
    vegetableJson[2020]["11月"],
    vegetableJson[2020]["12月"],
  ];
}
