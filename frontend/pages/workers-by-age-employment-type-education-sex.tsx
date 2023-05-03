import { CustomHead } from "../page_modules/components/CustomHead";
import { pageTitles } from "../page_modules/constants";
import { EmploymentChart } from "../page_modules/WorkersByAgeEmploymentTypeEducationSex/components/WorkersByAgeEmploymentTypeEducationSex";

const pageTitle = pageTitles["workers-by-age-employment-type-education-sex"];

export default function WorkersByAgeEmploymentTypeEducationSex() {
  return (
    <>
      <CustomHead pageTitle={pageTitle} />
      <h1 className="page_title">{pageTitle}</h1>
      <EmploymentChart />
      <div
        style={{ marginTop: "5rem", display: "flex", justifyContent: "center" }}
      >
        <div className="text_content">
          <h2>備考</h2>
          <p>
            2022年2月15日に公開された、労働力調査 詳細集計 全都道府県 全国 年次
            「年齢階級，教育，雇用形態別雇用者数(2002年～)」
            のデータより作成しています。グラフに用いているデータは下のリンクから閲覧できます。
          </p>
          <ul>
            <li>
              <a href="https://www.e-stat.go.jp/dbview?sid=0003006608">
                労働力調査 詳細集計　全都道府県 全国 年次1-5-1
                年齢階級，教育，雇用形態別雇用者数(2002年～) |
                統計表・グラフ表示 | 政府統計の総合窓口
              </a>
            </li>
          </ul>
          <p>
            労働力調査の概要、結果等の解説については下のリンクから閲覧できます。
          </p>
          <ul>
            <li>
              <a href="https://www.stat.go.jp/data/roudou/index2.html">
                統計局ホームページ/労働力調査　労働力調査の概要、結果等
              </a>
            </li>
          </ul>
        </div>
      </div>
    </>
  );
}
