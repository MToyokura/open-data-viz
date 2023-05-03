import { CommodityPriceChart } from "../page_modules/CommodityPrices/components/CommodityPriceChart";
import { CustomHead } from "../page_modules/components/CustomHead";
import { pageTitles } from "../page_modules/constants";

const pageTitle = pageTitles["commodity-prices"];

export default function WorkersByAgeEmploymentTypeEducationSex() {
  return (
    <>
      <CustomHead pageTitle={pageTitle} />
      <h1 className="page_title">{pageTitle}</h1>
      <CommodityPriceChart />
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div className="text_content">
          <h2>備考</h2>
          <p>
            小売物価統計調査（動向編）
            「主要品目の都市別小売価格－都道府県庁所在市及び人口15万以上の市(2000年1月～)」
            のデータより作成しています。グラフに用いているデータは下のリンクから閲覧できます。
          </p>
          <ul>
            <li>
              <a href="https://www.e-stat.go.jp/dbview?sid=0003421913">
                小売物価統計調査
                小売物価統計調査（動向編）主要品目の都市別小売価格－都道府県庁所在市及び人口15万以上の市(2000年1月～)
                | 統計表・グラフ表示 | 政府統計の総合窓口
              </a>
            </li>
          </ul>
          <p>
            小売物価統計調査の概要、結果等の解説については下のリンクから閲覧できます。
          </p>
          <ul>
            <li>
              <a href="https://www.stat.go.jp/data/kouri/doukou/indexf.html">
                統計局ホームページ/小売物価統計調査（動向編）関連情報
              </a>
            </li>
          </ul>
          <p>
            グラフを見るにあたって、時期によって価格の単位が異なる場合があるのでご注意ください。例えば、うるち米の単位は2002年1月に「10kg当たり」から「1袋辺り」に変更されています。このため見かけ上では2002年1月を堺にうるち米の価格が大幅に下がったように見えます。
          </p>
          <p>
            価格に使用されている単位は以下のリンク先にあるエクセルデータにて確認できます。
          </p>
          <ul>
            <li>
              <a href="https://www.stat.go.jp/data/kouri/doukou/3.html#meigara">
                統計局ホームページ/小売物価統計調査（動向編） 調査結果 #
                調査品目及び基本銘柄
              </a>
            </li>
          </ul>
        </div>
      </div>
    </>
  );
}
