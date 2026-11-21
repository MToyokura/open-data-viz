<script>
  import { VITE_ROOT_PATH } from "#lib/paths.js";
  import ArticleChart from "../ArticleChart.svelte";

  const tokyo = "13100";
  const videoAreas = ["13100", "01100", "27100", "40130"];
</script>

<svelte:head>
  <title>モノの価格の推移を見る | Open Data Viz</title>
  <meta
    name="description"
    content="小売物価統計調査（動向編）のデータから、身近な品目の価格推移を見ます。"
  />
</svelte:head>

<main class="page-content">
  <article class="article-content">
    <header class="article-header">
      <h1 class="article-title">モノの価格の推移を見る</h1>
    </header>

    <p>
      統計局の小売物価統計調査（動向編）のデータから個人的に興味のあったものをいくつかピックアップして見ていきたいと思います。
    </p>
    <p>自身で探索したい場合はこちらをご覧ください。</p>
    <a class="card" href={`${VITE_ROOT_PATH}/commodity_prices`}
      >主要品目の都市別小売価格</a
    >
    <p>
      本記事では基本的に東京都の特別区部（23区）の数値を見ていきます。表示期間は適当に切っています。
    </p>

    <section>
      <h2>1001 うるち米(単一原料米,「コシヒカリ」)</h2>
      <p>
        <strong>銘柄：</strong
        >国内産，精米，単一原料米（産地，品種及び産年が同一のもの），袋入り（５ｋｇ入り），「コシヒカリ」
      </p>
      <p><strong>データの連続性：</strong>2009年11月以降銘柄に変化なし</p>
      <ArticleChart
        series={[{ cat02Code: "01001", areaCode: tokyo }]}
        label="特別区部におけるコシヒカリの価格の推移"
      />
      <p>
        去年（2024年）の7月あたりから価格の上昇が続いていることがわかります。
      </p>
      <p>
        2023年ごろまでは東京23区で2300円前後でしたが、2025年4月の価格は4679円となっています。
      </p>
    </section>

    <section>
      <h2>1108 さんま</h2>
      <p><strong>銘柄：</strong>丸（長さ約２５ｃｍ以上）</p>
      <p>
        <strong>連続性：</strong
        >2025年1月から調査月が8月～11月に限定されるようになりました
      </p>
      <ArticleChart
        series={[{ cat02Code: "01108", areaCode: tokyo }]}
        label="特別区部におけるさんまの価格の推移"
      />
      <p>毎年8月に価格がピークになることがわかります。</p>
      <p>
        長期的な推移としては、2014年ごろから価格が上昇傾向にあることがわかります。
      </p>
      <p>
        同期間のさんまの漁獲量が気になったので図にしてみました。（出典：<a
          href="https://www.e-stat.go.jp/stat-search/files?tclass=000001024930&amp;cycle=0"
          >海面漁業魚種別漁獲量累年統計（全国）</a
        >）
      </p>
      <div class="separate-chart separate-chart--small">
        さんまの漁獲量の推移（別途対応）
      </div>
      <p>顕著な減少傾向となっています。</p>
    </section>

    <section>
      <h2>1341 鶏卵</h2>
      <p>
        <strong>銘柄：</strong
        >白色卵，パック詰（１０個入り），サイズ混合，〔卵重〕「ＭＳ５２ｇ～ＬＬ７６ｇ未満」，「ＭＳ５２ｇ～Ｌ７０ｇ未満」又は「Ｍ５８ｇ～Ｌ７０ｇ未満」
      </p>
      <p><strong>連続性：</strong>2018年1月以降銘柄に変化なし</p>
      <ArticleChart
        series={[{ cat02Code: "01341", areaCode: tokyo }]}
        label="特別区部における鶏卵の価格の推移"
      />
      <p>
        2023年に価格が上昇し、2024年に少し落ち着いたものの2025年に入り再び価格が上昇しています。
      </p>
    </section>

    <section>
      <h2>2136 牛丼(外食)</h2>
      <p><strong>銘柄：</strong>並，持ち帰りは除く</p>
      <p><strong>連続性：</strong>2019年1月に持ち帰りは対象外となりました</p>
      <ArticleChart
        series={[{ cat02Code: "02136", areaCode: tokyo }]}
        label="特別区部における牛丼の価格の推移"
      />
      <p>2013年の一時期においては牛丼1杯が280円でした。</p>
      <p>当時のニュース記事がネット上に残っていました。</p>
      <a
        class="news-card"
        href="https://www.nikkei.com/article/DGXNASGF1706K_Y3A410C1EB2000/"
        target="_blank"
        rel="noopener nofollow"
      >
        <strong
          >吉野家、牛丼並280円に値下げ　集客競争厳しく - 日本経済新聞</strong
        >
        <span
          >吉野家ホールディングス（HD）は18日午前、「吉野家」の牛丼並盛りの通常価格をこれまでより100円安い280円に値下げした</span
        >
        <small>www.nikkei.com</small>
      </a>
    </section>

    <section>
      <h2>9391 ビデオソフトレンタル料</h2>
      <p><strong>銘柄：</strong>ＤＶＤビデオ，旧作，洋画，７泊８日</p>
      <p><strong>連続性：</strong>2006年7月以降変化なし</p>
      <ArticleChart
        series={videoAreas.map((areaCode) => ({
          cat02Code: "09391",
          areaCode,
        }))}
        label="特別区部、札幌市、大阪市、福岡市におけるビデオソフトレンタル料の推移"
      />
      <p>
        地域差があって興味深かったので特別区部、札幌市、大阪市、福岡市を表示対象としています。
      </p>
      <p>
        2009年から2019年にかけて各地域で価格が下がっていることがわかります。
      </p>
    </section>

    <section>
      <h2>まとめ</h2>
      <p>
        記憶と照らし合わせながら見るとなかなかおもしろいのではないでしょうか。
      </p>
      <p>
        小売物価統計調査は他にもたくさんの品目のデータがありますが、後半の銘柄符号（番号）の大きいものは商品の選択肢の幅が広かったり、または銘柄変更によるデータの途切れがあったりと推移の分析には適さない感じでした。
      </p>
      <p>数値としては食料品が最も参考になるのではないかと思います。</p>
      <p>
        記事冒頭でも紹介しましたが、小売物価統計調査（動向編）のデータを見るサイトを作りましたのでご興味ある方はどうぞ。
      </p>
      <a class="card" href={`${VITE_ROOT_PATH}/commodity_prices`}
        >主要品目の都市別小売価格</a
      >
      <p>
        なお、データを見るときは品目の詳細とデータの連続性を確認するようにしましょう。
      </p>
      <p>
        ちなみにサイトは CC0
        で公開していますので自由に使っていただいて大丈夫です。（そもそもグラフに著作物性があるのかという話はありますが）
      </p>
      <p>
        これからもデータを更新したり、他のデータもグラフ化したりとサイトを充実させていきたいと思いますので応援いただけるとモチベーションにつながります🙇‍♀️
      </p>
    </section>
  </article>
</main>
