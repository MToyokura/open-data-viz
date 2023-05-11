import { CustomHead } from "../page_modules/components/CustomHead";
import { pageTitles } from "../page_modules/constants";

const pageTitle = pageTitles.about;

export default function About() {
  return (
    <>
      <CustomHead pageTitle={pageTitle} />
      <h1 className="page_title">{pageTitle}</h1>
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div className="width_640px">
          <h2>コンセプト</h2>
          <p>個人的に気になったデータを可視化していきます。</p>
          <h2>動機</h2>
          <p>
            これまでも気になった統計データを手元でグラフ化していたのですが、せっかくなので作ったものを公開していこうと思いこのサイトを作りました。
          </p>
          <p>
            世の中にはデータを可視化したものが数多く存在しますが、多くの場合それらが作られた過程を見ることができません。ここではデータの取得から加工までをすべて公開しようと考えています。その方が情報の信頼性が上がりますし、グラフひとつから得られる学びも増えるのではないかと思っています。
          </p>
          <h2>著作権</h2>
          <p>
            特に断りのない限り、本サイトのコンテンツは全て
            <a
              href="https://creativecommons.org/licenses/by/4.0/deed.ja"
              target="_blank"
              rel="noreferrer"
            >
              クリエイティブ・コモンズ 表示 4.0 ライセンス
            </a>
            のもとに提供されています。用途問わずご自由にお使いください。ソースコードは
            MIT ライセンスのもとに提供されています。 本サイトの GitHub
            レポジトリは下のリンクにあります。
          </p>
          <ul>
            <li>
              <a href="https://github.com/MToyokura/open-data-viz">
                MToyokura / open-data-viz
              </a>
            </li>
          </ul>
          <h2>間違いを見つけたら</h2>
          <p>GitHub の Issues よりお知らせください。</p>
          <h2>作者</h2>
          <p>
            このサイトは<a href="https://mtoyokura.github.io/">豊倉幹人</a>
            が、寛大な方々により無償で公開されているツールと、公費で賄われたデータを使って作っています。
          </p>
        </div>
      </div>
    </>
  );
}
