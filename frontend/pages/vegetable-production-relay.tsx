import { useRouter } from "next/router";
import { ParsedUrlQuery } from "querystring";
import { CustomHead } from "../page_modules/components/CustomHead";
import { pageTitles } from "../page_modules/constants";
import { VegetableProductionRelayPrep } from "../page_modules/VegetableProductionRelay/components/VegetableProductionRelayPrep";
import { vegetableSelect } from "../page_modules/VegetableProductionRelay/constants";
import { QueryParams } from "../page_modules/VegetableProductionRelay/types/QueryParams";

// エラーが出たのでメモ。
// Hydration failed because the initial UI does not match what was rendered on the server
// https://stackoverflow.com/questions/71706064/react-18-hydration-failed-because-the-initial-ui-does-not-match-what-was-render

const pageTitle = pageTitles["vegetable-production-relay"];

const defaultQueryParams: QueryParams = {
  yearMonthIndex: 0,
  circleSize: 5,
  vegetable: "だいこん",
};

function constructQueryParams(parsedUrlQuery: ParsedUrlQuery): QueryParams {
  let params = { ...defaultQueryParams };
  const urlQueryYearMonthIndex = parsedUrlQuery["yearMonthIndex"];
  const urlQueryCircleSize = parsedUrlQuery["circleSize"];
  const urlQueryVegetable = parsedUrlQuery["vegetable"];
  if (
    urlQueryYearMonthIndex &&
    Number(urlQueryYearMonthIndex) > 0 &&
    Number(urlQueryYearMonthIndex) < 36
  ) {
    params["yearMonthIndex"] = Number(urlQueryYearMonthIndex);
  }
  if (urlQueryCircleSize && Number(urlQueryCircleSize) > 0) {
    params["circleSize"] = Number(urlQueryCircleSize);
  }
  if (
    urlQueryVegetable &&
    vegetableSelect.find((element) => {
      return element === urlQueryVegetable;
    })
  ) {
    params["vegetable"] = urlQueryVegetable as string;
  }
  return params;
}

export default function VegetableProductionRelay() {
  // https://github.com/vercel/next.js/discussions/11484#discussioncomment-356055
  const { isReady: queryIsReady, query: parsedUrlQuery } = useRouter();

  return (
    <>
      <CustomHead pageTitle={pageTitle} />
      <h1 className="page_title">{pageTitle}</h1>
      {queryIsReady ? (
        <VegetableProductionRelayPrep
          queryParams={constructQueryParams(parsedUrlQuery)}
        />
      ) : (
        // SSR を使わないので && で問題ないけど dev 環境で React Hydration Error が出る。
        // suppressHydrationWarning={true} をつけてみたけどエラーが消えないので
        // ここに書き出している。
        <div style={{ display: "flex", justifyContent: "center" }}>
          loading...
        </div>
      )}
      <div
        style={{ marginTop: "5rem", display: "flex", justifyContent: "center" }}
      >
        <div className="text_content">
          <h2>備考</h2>
          <p>
            図に表示される数値は、全国の主要な青果物卸売市場における青果物の卸売数量の合計のうち、
            各都道府県が産地となっている数量を表しています。
            そのため青果物卸売市場調査の対象でない卸売市場や、
            <a href="http://lib.ruralnet.or.jp/nrpd/#koumoku=12184">市場外</a>
            で流通した青果物は含まれていません。
          </p>
          <p>
            卸売数量には青果物卸売市場調査（産地別）のデータを使用しています。
            例えば2020年のデータは下のリンクから見ることができます。
          </p>
          <ul>
            <li>
              <a href="https://www.e-stat.go.jp/stat-search/database?page=1&layout=datalist&toukei=00500226&tstat=000001015623&cycle=7&tclass1=000001020455&tclass2=000001158847&cycle_facet=tclass1%3Atclass2&tclass3val=0">
                青果物卸売市場調査 確報 令和２年青果物卸売市場調査（産地別） |
                データベース | 統計データを探す | 政府統計の総合窓口
              </a>
            </li>
          </ul>
          <p>青果物卸売市場調査の概要は下のリンクから見ることができます。</p>
          <ul>
            <li>
              <a href="https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/gaiyou/index.html">
                青果物卸売市場調査の概要：農林水産省
              </a>
            </li>
          </ul>
          <p>
            地図データには Humanitarian Data Exchange (HDX)
            が公開しているデータを加工したものを使用しています。元データは
            Creative Commons Attribution for Intergovernmental Organisations
            のもとに公開されており、 Contributor は OCHA Regional Office for
            Asia and the Pacific (ROAP)
            です。元データは下のリンクから入手することができます。
          </p>
          <ul>
            <li>
              <a href="https://data.humdata.org/dataset/cod-ab-jpn">
                Japan - Subnational Administrative Boundaries - Humanitarian
                Data Exchange
              </a>
            </li>
          </ul>
        </div>
      </div>
    </>
  );
}
