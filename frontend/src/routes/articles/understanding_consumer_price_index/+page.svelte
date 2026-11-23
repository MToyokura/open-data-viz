<script lang="ts">
  import LoadState from "#lib/LoadState.svelte";
  import MathFormula from "#lib/MathFormula.svelte";
  import type { EChartsOption } from "echarts";
  import { onMount } from "svelte";
  import {
    fetchLookups,
    fetchSeries,
  } from "../../consumer_price_index/utils/fetchData.js";
  import { transformData } from "../../consumer_price_index/utils/transformData.js";
  import ArticleHeading from "../ArticleHeading.svelte";
  import CpiArticleChart from "../consumer_price_index/CpiArticleChart.svelte";

  const weightedExample = String.raw`\frac{80 \times 6 + 120 \times 3 + 115 \times 1}{6 + 3 + 1} = 95.5`;

  const title = "消費者物価指数を理解する";
  const chartLabel = "全国の生鮮食品及びエネルギーを除く総合（原指数）";
  let chartRequest = $state<ReturnType<typeof loadChart>>();

  async function loadChart() {
    const itemCode = "0178";
    const areaCode = "00000";
    const { items } = await fetchLookups();
    const itemIndex = items.findIndex(
      (item: { code: string }) => item.code === itemCode,
    );
    if (itemIndex < 0) return null;

    const series = await fetchSeries({
      itemCode,
      areaCode,
      itemIndex,
      itemCount: items.length,
    });
    if (
      !series?.timeValuePairs?.some(
        ([, value]) => typeof value === "number" && Number.isFinite(value),
      )
    ) {
      return null;
    }

    return transformData(
      [{ ...series, itemName: items[itemIndex].name, areaName: "全国" }],
      "index",
    ).chartData;
  }

  onMount(() => {
    chartRequest = loadChart();
  });

  let ingredients = $state([
    { name: "米", priceChange: -20, weight: 6 },
    { name: "牛肉", priceChange: 20, weight: 3 },
    { name: "カレールウ", priceChange: 15, weight: 1 },
  ]);
  const totalWeight = 10;
  const weightedIndex = $derived(
    ingredients.reduce(
      (sum, { priceChange, weight }) => sum + (100 + priceChange) * weight,
      0,
    ) / totalWeight,
  );

  const weightedIndexChartLabel = "各品目の加重指数と全体の指数";
  const weightedIndexChart = $derived<EChartsOption>({
    animation: false,
    aria: { enabled: true },
    tooltip: { trigger: "axis" },
    legend: { bottom: 8, selectedMode: false },
    grid: { left: 24, right: 24, top: 16, bottom: 80 },
    xAxis: {
      type: "value",
      min: 0,
      max: Math.max(100, Math.ceil(weightedIndex / 20) * 20),
      name: "指数",
      nameLocation: "middle",
      nameGap: 25,
    },
    yAxis: { type: "category", data: ["全体"], show: false },
    series: ingredients.map(({ name, weight, priceChange }) => ({
      type: "bar",
      name,
      stack: "index",
      barMaxWidth: 36,
      data: [((100 + priceChange) * weight) / totalWeight],
    })),
  });

  let lastAdjustedIngredient: string | undefined;
  let firstRemainingShare = 0.5;

  function setWeight(name: string, requestedWeight: number) {
    if (!Number.isFinite(requestedWeight)) return;
    const weight = Math.max(
      0,
      Math.min(totalWeight, Math.round(requestedWeight)),
    );
    const selected = ingredients.find((ingredient) => ingredient.name === name);
    const [first, second] = ingredients.filter(
      (ingredient) => ingredient.name !== name,
    );
    if (!selected || !first || !second || selected.weight === weight) return;

    // Keep the ratio stable across input events from the same slider.
    if (lastAdjustedIngredient !== name) {
      const previousRemainder = first.weight + second.weight;
      firstRemainingShare =
        previousRemainder > 0 ? first.weight / previousRemainder : 0.5;
      lastAdjustedIngredient = name;
    }

    const remainder = totalWeight - weight;
    selected.weight = weight;
    first.weight = Math.round(remainder * firstRemainingShare);
    second.weight = remainder - first.weight;
  }

  const baseTotal = 1000;
  const currentTotal = $derived((baseTotal * weightedIndex) / 100);
  const displayedIndex = $derived(weightedIndex.toFixed(1));
  const amountFormat = new Intl.NumberFormat("ja-JP", {
    maximumFractionDigits: 2,
  });

  function formatAmount(amount: number) {
    return `${amountFormat.format(amount)}円`;
  }

  const percentFormat = new Intl.NumberFormat("ja-JP", {
    signDisplay: "exceptZero",
    maximumFractionDigits: 1,
  });

  function formatChange(change: number) {
    return `${percentFormat.format(change)}％`;
  }

  function getBaseAmount(weight: number) {
    return (baseTotal * weight) / totalWeight;
  }

  function getCurrentAmount(weight: number, priceChange: number) {
    return (getBaseAmount(weight) * (100 + priceChange)) / 100;
  }

  function formatFormulaAmount(amount: number) {
    return String(amount).replace(/\B(?=(\d{3})+(?!\d))/g, "{,}");
  }

  const currentAmountSum = $derived(
    ingredients
      .map(({ weight, priceChange }) =>
        formatFormulaAmount(getCurrentAmount(weight, priceChange)),
      )
      .join(" + "),
  );
  const totalFormula = $derived(
    String.raw`\frac{${currentAmountSum}}{${formatFormulaAmount(baseTotal)}} \times 100 = ${displayedIndex}`,
  );
  const weightedIndexFormula = $derived(
    ingredients
      .map(
        ({ weight, priceChange }) =>
          String.raw`${weight / totalWeight} \times ${100 + priceChange}`,
      )
      .join(" + ") + ` = ${displayedIndex}`,
  );
</script>

<svelte:head>
  <title>{title} | Open Data Viz</title>
</svelte:head>

<main class="page-content">
  <article class="article-content">
    <header class="article-header">
      <ArticleHeading id="article-title" {title} level="h1" />
      <dl class="article-dates">
        <div>
          <dt>作成日時</dt>
          <dd>
            <time datetime="2026-11-23T10:10:00+09:00"
              >2026年11月23日 10:10</time
            >
          </dd>
        </div>
        <div>
          <dt>更新日時</dt>
          <dd>
            <time datetime="2026-11-23T10:10:00+09:00"
              >2026年11月23日 10:10</time
            >
          </dd>
        </div>
      </dl>
    </header>
    <p>
      このあたりを AI
      に放り込んで解説をしてもらうのが最も効果的な学び方かもしれません。
    </p>
    <ul>
      <li>
        <a href="https://www.stat.go.jp/data/cpi/4-1.html"
          >消費者物価指数に関するQ&A（回答）</a
        >
      </li>
      <li>
        <a href="https://www.stat.go.jp/data/cpi/2025/mikata/pdf/0.pdf"
          >消費者物価指数のしくみと見方（PDF）</a
        >
      </li>
      <li>
        <a href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf"
          >2025年基準 消費者物価指数の解説（PDF）</a
        >
      </li>
    </ul>
    <p>
      一方で全体の概説のような資料を読むのもまた学びにつながりますし、私の学びにもなるので書いていきます。
    </p>
    <section>
      <ArticleHeading id="cpi-definition" title="消費者物価指数とは" />
      <blockquote>
        <p>
          消費者物価指数は、全国の世帯が購入する財及びサービスの価格変動を総合的に測定し、物価の変動を時系列的に測定するものである。すなわち、消費者物価指数は、家計の消費構造を一定のものに固定し、これに要する費用が物価の変動によってどう変化するかを指数値で示したものである。
        </p>
      </blockquote>
      <p>
        <a href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf#page=9"
          >総務省統計局『2025年基準 消費者物価指数の解説』p.1</a
        >
      </p>
      <p>
        消費者物価指数には、個々の品目の価格指数と、複数の品目をまとめた指数があります。ニュースでよく見る「生鮮食品及びエネルギーを除く総合」は後者に当たります。
      </p>
      <figure class="article-chart">
        {#if chartRequest}
          {#await chartRequest}
            <LoadState state="loading" message="データを読み込んでいます…" />
          {:then chartData}
            {#if chartData}
              <CpiArticleChart {chartData} label={chartLabel} />
            {:else}
              <p role="status">表示できるデータがありません。</p>
            {/if}
          {:catch}
            <LoadState
              state="error"
              message="データを取得できませんでした。"
              onretry={() => (chartRequest = loadChart())}
            />
          {/await}
        {:else}
          <LoadState state="loading" message="データを読み込んでいます…" />
        {/if}
        <figcaption>{chartLabel}（2025年＝100）</figcaption>
      </figure>
    </section>
    <section>
      <ArticleHeading id="base-year" title="「2025年基準」とは" />
      <blockquote>
        <p>
          消費者物価指数は、基準となる年の物価を100として、その時々の物価を比較計算した数値となっています。この、物価の基準となる時点のことを「指数の基準時」と呼んでいます。
        </p>
        <p>
          また、消費者物価指数は、ある時点の消費構造（品目ごとの支出割合）をウエイトとして、個々の品目の価格指数を加重平均して算出しています。この、ウエイトに採用した年次のことを、「ウエイトの参照年次」と呼んでいます。
        </p>
        <p>
          現在、日本の消費者物価指数のうち毎月公表している公式系列については、「指数の基準時」と「ウエイトの参照年次」を共に2025年の1年間としており、これを「基準年」と呼んでいます。
        </p>
        <p>
          なお、基準年は、西暦年の末尾が0と5の年を基準時として、5年ごとに改定（基準改定）しています。その際、併せて指数に採用する品目などの見直しも行っています。
        </p>
      </blockquote>
      <p>
        <a href="https://www.stat.go.jp/data/cpi/4-1.html"
          >総務省統計局「消費者物価指数に関するQ&A（回答）」A-3</a
        >
      </p>
      <p>
        「2025年基準」とは2025年の年平均指数を100とします、ということです。各品目・市町村の基準時価格は、原則として2025年1月から12月までの比較時価格を単純平均して求めます。
      </p>
      <p>
        <a
          href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf#page=30"
          >総務省統計局『2025年基準 消費者物価指数の解説』p.22</a
        >
      </p>
    </section>
    <section>
      <ArticleHeading id="weights" title="「ウエイト」とは" />
      <p>
        「ウエイト」とは家計の消費支出に占める割合であり、指数の計算において使用されます。
      </p>
    </section>
    <section>
      <ArticleHeading id="calculation" title="実際に指数を計算してみる" />
      <p>
        <a href="https://www.stat.go.jp/data/cpi/2025/mikata/pdf/0.pdf#page=20"
          >「消費者物価指数のしくみと見方」の p.14</a
        >
        にあるカレーの例をそのまま使います。カレーを例に、米、牛肉、カレールウの3品目への支出の合計を、家計の消費支出全体に見立てます。それぞれの具材が指数品目に対応します。
      </p>
      <blockquote>
        <p>
          米、牛肉及びカレールウの３品目によって物価指数を作成するとします。今月の価格が基準時に比べて、米が
          20％値下がりして、基準時の 100 に対して 80 に、一方、牛肉は
          20％値上がりして 120 に、カレールウも 15％値上がりして 115
          になったとします。これを単純に平均すると、
        </p>
        <MathFormula display tex={String.raw`\frac{80 + 120 + 115}{3} = 105`} />
        <p>
          となり、基準時の 100
          に対して５％上昇したと計算されます。しかし、家計の消費支出上、この３品目に対する重要度は必ずしも同じではありません。この３品目の支出額の割合が、米６、牛肉３、カレールウ１であったとします。そこでこれらの値動きを、ウエイトを加味して計算すると、
        </p>
        <MathFormula display tex={weightedExample} />
        <p>
          となります。単純に計算した場合に比べてウエイトの大きさが反映され、4.5％の下落になりました。各品目の全体に占める支出額の割合を加味する、つまり、ウエイトを付けて平均する計算方法を「加重平均」といいます。
        </p>
      </blockquote>
      <p>
        <a href="https://www.stat.go.jp/data/cpi/2025/mikata/pdf/0.pdf#page=20"
          >総務省統計局『消費者物価指数のしくみと見方』pp.14–15</a
        >
      </p>
      <p>
        引用の例では、基準時の支出額の比が米、牛肉、カレールウの順に6:3:1であり、この比がウエイトです。たとえば、基準時に米に600円、牛肉に300円、カレールウに100円を支出したとします。今月も同じ量を買うと、上の価格変化を反映した支出額は、米480円、牛肉360円、カレールウ115円です。合計は955円なので、基準時の1,000円を100とした指数は次のようになります。
      </p>
      <MathFormula
        display
        tex={String.raw`\frac{955}{1{,}000} \times 100 = 95.5`}
      />
      <p>このとき、今月の支出に占める米の割合は次のとおりです。</p>
      <MathFormula display tex={String.raw`\frac{480}{955} \approx 50.3\%`} />
      <p>
        米の値下がりとほかの品目の値上がりにより、米の支出割合は基準時の60％から今月の約50.3％に下がりました。しかしこの50.3％は指数の計算には使いません。消費者物価指数の計算には基準時の支出比
        6:3:1
        を使うので、米のウエイトは6のままです。下記のシミュレーションでウエイトを変えると、基準時の支出比が異なるケースを試すことができます。
      </p>
      <p>基準時からの価格変化とウエイトを変えて、加重平均を見てみましょう。</p>
      <ArticleHeading
        id="simulation"
        title="価格とウエイトのシミュレーション"
        level="h3"
      />
      <fieldset class="simulation" aria-labelledby="simulation">
        <p>
          基準時の合計を{formatAmount(
            baseTotal,
          )}とし、今月も同じ量を買う場合の金額を表示します。
        </p>
        <p>
          このシミュレーションでは、ウエイトの合計を{totalWeight}に固定しています。1品目のウエイトを変えると、残り2品目のウエイトも自動で調整されます。
        </p>
        {#each ingredients as ingredient (ingredient.name)}
          <div class="ingredient">
            <strong
              >{ingredient.name}（指数 {100 + ingredient.priceChange}）</strong
            >
            <label>
              <span>基準時からの価格変化率</span>
              <input
                type="range"
                min="-100"
                max="100"
                step="1"
                aria-label={`${ingredient.name}の基準時からの価格変化率`}
                aria-valuetext={formatChange(ingredient.priceChange)}
                bind:value={ingredient.priceChange}
              />
              <span>{formatChange(ingredient.priceChange)}</span>
            </label>
            <label>
              <span>基準時のウエイト</span>
              <input
                type="range"
                min="0"
                max={totalWeight}
                step="1"
                aria-label={`${ingredient.name}の基準時ウエイト`}
                aria-valuetext={`合計${totalWeight}のうち${ingredient.weight}`}
                bind:value={
                  () => ingredient.weight,
                  (weight) => setWeight(ingredient.name, weight)
                }
              />
              <span>{ingredient.weight}</span>
            </label>
            <div>
              基準時 {formatAmount(getBaseAmount(ingredient.weight))} → 今月 {formatAmount(
                getCurrentAmount(ingredient.weight, ingredient.priceChange),
              )}
            </div>
          </div>
        {/each}
        <p>
          <output>
            合計 {formatAmount(baseTotal)} → {formatAmount(currentTotal)}
            加重平均 {displayedIndex}（基準時比 {formatChange(
              weightedIndex - 100,
            )}）
          </output>
        </p>
        <MathFormula display tex={totalFormula} />
        <MathFormula display tex={weightedIndexFormula} />
        <figure class="article-chart weighted-index-chart">
          <CpiArticleChart
            chartData={weightedIndexChart}
            label={weightedIndexChartLabel}
          />
          <figcaption>
            棒の各区間は、品目別指数に基準時の支出割合を掛けた値です。各区間の値を合計すると、全体の指数になります。
          </figcaption>
        </figure>
      </fieldset>
    </section>
    <section>
      <ArticleHeading id="laspeyres" title="ラスパイレス式" />
      <p>
        先ほどのカレーの例で、各品目の指数を基準時の支出比6:3:1で加重平均しました。実は、そのときに使った次の式がラスパイレス式です。
      </p>
      <MathFormula display tex={weightedExample} />
      <p>
        総務省統計局の<a
          href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf#page=36"
          >『2025年基準 消費者物価指数の解説』（PDF）</a
        >には、「指数算式」という用語が出てきます。これは、品目ごとの価格から全体の指数を求める計算式のことです。この資料では、消費者物価指数の計算に「基準時加重相対法算式（ラスパイレス型）」を用いるとしています。先ほどカレーの例で使ったラスパイレス式を一般化すると、次のように書けます。
      </p>
      <MathFormula
        display
        tex={String.raw`
          I_t = \frac{\sum\limits_{i=1}^n p_{t,i}q_{0,i}}{\sum\limits_{i=1}^n p_{0,i}q_{0,i}} \times 100 =
            \frac{\sum\limits_{i=1}^n \frac{p_{t,i}}{p_{0,i}}w_{0,i}}{\sum\limits_{i=1}^n w_{0,i}} \times 100
        `}
      />
      <p>
        一般式の左側では、基準時と同じ量を買うのにかかる費用を、今月と基準時の価格で比べています。一般式の右側は、各品目の価格比を基準時の支出額で加重平均する形です。ウエイト
        <MathFormula tex={"w_{0,i} = p_{0,i}q_{0,i}"} /> を代入すると、2つの形が一致します。
      </p>
      <ul>
        <li>
          <MathFormula tex="I_t" /> は比較時の全体の指数で、この例では今月の95.5です。
        </li>
        <li>
          <MathFormula tex="0" /> は基準時、<MathFormula tex="t" /> は比較時（今月）です。
        </li>
        <li>
          <MathFormula tex="i" />
          は品目を表し、1が米、2が牛肉、3がカレールウです。<MathFormula
            tex="n"
          />
          は品目数で、この例では3です。<MathFormula tex={String.raw`\sum`} /> は3品目分を足し合わせることを表します。
        </li>
        <li>
          <MathFormula tex={"p_{0,i}"} />
          と
          <MathFormula tex={"p_{t,i}"} />
          は、それぞれ基準時と今月の各品目の単価です。個々の単価は例に示されていませんが、価格比
          <MathFormula tex={String.raw`\frac{p_{t,i}}{p_{0,i}}`} /> は米が0.80、牛肉が1.20、カレールウが1.15です。
        </li>
        <li>
          <MathFormula tex={"q_{0,i}"} /> は各品目の基準時の購入数量です。具体的な数量は例に示されていませんが、今月の費用を計算するときにも同じ数量を使います。
        </li>
        <li>
          <MathFormula tex={"w_{0,i}"} /> は基準時の支出額（単価×購入数量）です。米、牛肉、カレールウの順に600円、300円、100円で、比にすると6:3:1です。
        </li>
      </ul>
      <p>
        まず、一般式の左側をカレーの例に当てはめます。基準時の購入数量を固定すると、今月の費用は、品目ごとに基準時の支出額に今月の価格比を掛けて求められます。たとえば米（品目1）は、基準時に600円分買った量を今月も買うと、単価が20％下がって基準時の0.80倍になっているため、費用も600円の0.80倍になります。米、牛肉、カレールウの順に、金額は次のとおりです（単位は円）。
      </p>
      <MathFormula
        display
        tex={String.raw`
          \begin{aligned}
            p_{t,1}q_{0,1} &= 0.80 \times 600 = 480 \\
            p_{t,2}q_{0,2} &= 1.20 \times 300 = 360 \\
            p_{t,3}q_{0,3} &= 1.15 \times 100 = 115
          \end{aligned}
        `}
      />
      <p>総和を3品目分に展開して金額を代入します。</p>
      <MathFormula
        display
        tex={String.raw`
          \begin{aligned}
            I_t &= \frac{p_{t,1}q_{0,1} + p_{t,2}q_{0,2} + p_{t,3}q_{0,3}}{p_{0,1}q_{0,1} + p_{0,2}q_{0,2} + p_{0,3}q_{0,3}} \times 100 \\
                &= \frac{480 + 360 + 115}{600 + 300 + 100} \times 100 \\
                &= \frac{480 + 360 + 115}{1{,}000 / 100} \\
                &= \frac{955}{10} \\
                &= 95.5
          \end{aligned}
        `}
      />
      <p>
        一般式の右側で使う「価格比」は、各品目の今月の単価を基準時の単価で割った値です。品目別の指数は基準時を100としているので、指数を100で割ると価格比になります。たとえば、米の指数80は、今月の単価が基準時の0.80倍であることを示しています。米、牛肉、カレールウの順に、価格比は次のとおりです。
      </p>
      <MathFormula
        display
        tex={String.raw`
          \begin{aligned}
            \frac{p_{t,1}}{p_{0,1}} &= \frac{80}{100} = 0.80 \\
            \frac{p_{t,2}}{p_{0,2}} &= \frac{120}{100} = 1.20 \\
            \frac{p_{t,3}}{p_{0,3}} &= \frac{115}{100} = 1.15
          \end{aligned}
        `}
      />
      <p>この価格比と基準時の支出額を、一般式の右側に代入します。</p>
      <MathFormula
        display
        tex={String.raw`
          \begin{aligned}
            I_t &= \frac{\frac{p_{t,1}}{p_{0,1}}w_{0,1} + \frac{p_{t,2}}{p_{0,2}}w_{0,2} + \frac{p_{t,3}}{p_{0,3}}w_{0,3}}{w_{0,1} + w_{0,2} + w_{0,3}} \times 100 \\
                &= \frac{0.80 \times 600 + 1.20 \times 300 + 1.15 \times 100}{600 + 300 + 100} \times 100 \\
                &= \frac{80 \times 6 + 120 \times 3 + 115 \times 1}{6 + 3 + 1} \\
                &= \frac{480 + 360 + 115}{10} \\
                &= \frac{955}{10} \\
                &= 95.5
          \end{aligned}
        `}
      />
    </section>
    <section>
      <ArticleHeading id="weight-calculation" title="ウエイトの作成" />
      <p>
        上記のカレーの例では、米、牛肉、カレールウの基準時の支出額の比を6:3:1とし、その割合を使って品目別の指数を加重平均しました。実際の消費者物価指数でも基準年の支出をもとにウエイトを作るという考え方は同じですが、家計調査の支出額をそのまま使うわけではなく、品目の区分や地域ごとの消費の違いなどを考慮して調整が行われます。統計局の解説では、基本分類ウエイトの作成について、家計調査市町村別ウエイト、指数市町村別ウエイト、生鮮食品の月別ウエイトの作成手順を説明しています。
      </p>
      <ul>
        <li>
          <strong>家計調査市町村別ウエイト</strong><br />
          家計調査の市町村別の支出額を、消費者物価指数で使う品目の区分に対応させて作ります。
        </li>
        <li>
          <strong>指数市町村別ウエイト</strong><br />
          家計調査から作ったウエイトを価格調査の対象となる市町村に対応させ、一部の品目について地域の実態に合わせた補正を行い、その市町村が代表する世帯数を反映して作ります。
        </li>
        <li>
          <strong>生鮮食品の月別ウエイト</strong><br />
          生鮮魚介・生鮮野菜・生鮮果物は、季節によって購入される品目や数量が大きく変わるため、個々の品目のウエイトが月ごとに設定されています。ただし、「生鮮野菜」などの分類全体のウエイト（類ウエイト）は、年間を通じて固定されています。例えば、生鮮野菜全体のウエイトは変わらないが、その内訳であるキャベツやトマトなどのウエイトは月ごとに変わる、という具合です。2025年基準では、2024年と2025年の購入数量をもとに、あらかじめ月別のウエイトを作成しています。
        </li>
      </ul>
      <p>
        カレーの例で計算したのは品目別の指数を加重平均する部分です。実際には、同じ品目でも市町村によって価格の動きが異なります。全国の品目別指数を求めるとき、市町村ごとに指数を算出する品目は、各調査市町村の品目別ウエイトで加重平均します。一方、後で紹介するドーナツ（外食）のように、同じ品目別指数を全国で使う品目もあります。こうして得た全国の品目別指数を全国の品目別ウエイトを使って加重平均する計算を、品目の小分類、中分類、10大費目へと段階的にまとめ、最終的に総合指数を求めます。
      </p>
      <p>
        詳しくは、総務省統計局の<a
          href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf#page=31"
          >「2025年基準 消費者物価指数の解説」の「第4 ウエイト（W₀）の作成」</a
        >をご参照ください。
      </p>
    </section>
    <section>
      <ArticleHeading id="seasonal-adjustment" title="季節調整" />
      <p>
        消費者物価指数には、毎年同じ時期に繰り返される季節的な値動き（季節変動）が含まれています。
      </p>
      <p>
        季節変動の影響を除いて物価の動きを見る方法の一つに、前年同月比があり、これを使うと同じ月同士を比較することで季節変動の影響を受けにくくなります。ただし、前年同月比には直近の物価の動きだけでなく、1年前の物価の動きも影響します。
      </p>
      <p>
        もう一つの方法が季節調整です。季節調整では、指数に含まれる季節変動を推定し、その影響を取り除きます。季節調整済指数を使うと、前月との比較によって直近の物価変動を捉えることができます。
      </p>
      <p>
        総務省統計局は、全国について8系列、東京都区部について5系列の季節調整済指数を作成しています。
      </p>
      <ul>
        <li>
          <strong>基本分類指数（全国・東京都区部）</strong
          >：総合、生鮮食品を除く総合、生鮮食品及びエネルギーを除く総合
        </li>
        <li>
          <strong>財・サービス分類指数（全国・東京都区部）</strong
          >：財、サービス
        </li>
        <li>
          <strong>ラスパイレス連鎖基準方式による指数（全国のみ）</strong
          >：総合、生鮮食品を除く総合、生鮮食品及びエネルギーを除く総合
        </li>
      </ul>
      <p>
        このように、季節調整はすべての品目別指数に行われるわけではありません。
      </p>
      <p>
        詳しくは、総務省統計局の<a
          href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf#page=48"
          >「2025年基準 消費者物価指数の解説」の「第7 季節調整」</a
        >をご参照ください。
      </p>
    </section>
    <section>
      <ArticleHeading id="model-items" title="モデル品目の計算をやってみる" />
      <ArticleHeading id="donuts" title="2163 ドーナツ（外食）" level="h3" />
      <figure class="article-chart">
        <table class="model-item-table">
          <caption>2163 ドーナツ（外食）</caption>
          <thead>
            <tr
              ><th scope="col">品目</th><th scope="col">銘柄</th><th scope="col"
                >単位</th
              ></tr
            >
          </thead>
          <tbody>
            <tr
              ><th scope="row">ドーナツ（外食）</th><td
                >飲食のできる店におけるドーナツ<br />商品別</td
              ><td>1個</td></tr
            >
            <tr
              ><td colspan="3"
                ><strong>価格選定</strong>
                <ol>
                  <li>
                    主にドーナツを販売している外食店について、売上高の大きい代表的な企業を選定する。
                  </li>
                  <li>
                    選定した企業において通年販売される商品のうち、販売数量の多い代表的な商品を複数選定する。
                  </li>
                </ol></td
              ></tr
            >
            <tr
              ><td colspan="3"
                ><strong>指数算出方法</strong>
                <p>
                  商品（<i>a</i>）別の販売数量（<i>q</i
                  >）を用いて、品目別価格指数を算出する。
                </p>
                <MathFormula
                  display
                  tex={String.raw`I_t = \frac{\sum_a p_{t,a}q_{0,a}}{\sum_a p_{0,a}q_{0,a}} \times 100`}
                />
                <p><i>t</i>：比較時、0：基準時、<i>a</i>：商品</p></td
              ></tr
            >
            <tr
              ><td colspan="3"><strong>価格指数の適用</strong><br />全国一律</td
              ></tr
            >
          </tbody>
        </table>
        <figcaption>
          ドーナツ（外食）の価格選定と指数算出方法<br />
          <a
            href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf#page=52"
            >総務省統計局『2025年基準 消費者物価指数の解説』p.44</a
          >
        </figcaption>
      </figure>
      <p>
        代表的な商品を2つ選んだとします。基準時に商品Aは1個100円で300個、商品Bは1個150円で200個売れました。今月の価格は、商品Aが110円、商品Bが180円です。
      </p>
      <p>
        基準時と同じ個数を買うと、基準時には60,000円、今月は69,000円かかります。
      </p>
      <MathFormula
        display
        tex={String.raw`\frac{110 \times 300 + 180 \times 200}{100 \times 300 + 150 \times 200} \times 100 = 115`}
      />
      <p>
        基準時を100とした指数は115で、15％の上昇です。この品目の指数は全国一律に適用されます。
      </p>
      <ArticleHeading id="private-rent" title="3000 民営家賃" level="h3" />
      <figure class="article-chart">
        <table class="model-item-table">
          <caption>3000 民営家賃</caption>
          <thead>
            <tr
              ><th scope="col">品目</th><th scope="col">銘柄</th><th scope="col"
                >単位</th
              ></tr
            >
          </thead>
          <tbody>
            <tr
              ><th scope="row">民営家賃</th><td
                >民営借家の家賃<br />延面積区分別、構造別</td
              ><td>1か月</td></tr
            >
            <tr
              ><td colspan="3"
                ><strong>価格選定</strong>
                <ol>
                  <li>
                    調査世帯（各調査市町村内に設けた家賃調査地区に居住する全ての民営借家世帯）の家賃を選定する。
                  </li>
                  <li>
                    調査価格を、延面積及び構造により、以下の4区分に分類する。
                    <ul>
                      <li>木造小住宅：延面積30 m²未満、かつ木造</li>
                      <li>木造中住宅：延面積30 m²以上、かつ木造</li>
                      <li>非木造小住宅：延面積30 m²未満、かつ非木造</li>
                      <li>非木造中住宅：延面積30 m²以上、かつ非木造</li>
                    </ul>
                  </li>
                  <li>
                    小売物価統計調査の家賃調査は、調査市町村別に家賃調査地区が3群に分けられ、毎月1群ずつ各群の民営借家世帯を調査するため、当月調査対象でない民営借家世帯は、直近の調査価格を用いる。
                  </li>
                  <li>
                    転出により当月空き家となった民営借家世帯は、一定の期間は、直近の調査した家賃と延面積を当月分として代入する。
                  </li>
                </ol></td
              ></tr
            >
            <tr
              ><td colspan="3"
                ><strong>指数算出方法</strong>
                <ol>
                  <li>
                    世帯（<i>b</i>）別の家賃及び延面積（<i>A</i
                    >）を用いて、区分（<i>a</i>）別の平均価格（3.3
                    m²当たり）を算出する。
                    <MathFormula
                      display
                      tex={String.raw`P_{t,a} = \frac{\sum_b p_{t,a,b}}{\sum_b A_{t,a,b}} \times 3.3`}
                    />
                    <MathFormula
                      display
                      tex={String.raw`{}= \frac{\sum_{b_1 \in \text{当月調査群}} p_{t,a,b_1} + \sum_{b_2 \in \text{前月調査群}} p_{t,a,b_2} + \sum_{b_3 \in \text{前々月調査群}} p_{t,a,b_3}}{\sum_{b_1 \in \text{当月調査群}} A_{t,a,b_1} + \sum_{b_2 \in \text{前月調査群}} A_{t,a,b_2} + \sum_{b_3 \in \text{前々月調査群}} A_{t,a,b_3}} \times 3.3`}
                    />
                  </li>
                  <li>
                    基準時価格で除して、区分別の指数を算出する。
                    <MathFormula
                      display
                      tex={String.raw`I_{t,a} = \frac{P_{t,a}}{P_{0,a}} \times 100`}
                    />
                  </li>
                  <li>
                    区分別の構成比（<i>w</i
                    >）を用いて加重平均し、調査市町村別の品目別価格指数を算出する。
                    <MathFormula
                      display
                      tex={String.raw`I_t = \frac{\sum_a I_{t,a}w_{0,a}}{\sum_a w_{0,a}}`}
                    />
                  </li>
                </ol>
                <p>
                  <i>t</i>：比較時、0：基準時、<i>a</i>：区分、<i>b</i>：世帯
                </p></td
              ></tr
            >
            <tr
              ><td colspan="3"
                ><strong>価格指数の適用</strong><br />調査市町村別</td
              ></tr
            >
          </tbody>
        </table>
        <figcaption>
          民営家賃の価格選定と指数算出方法<br />
          <a
            href="https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf#page=53"
            >総務省統計局『2025年基準 消費者物価指数の解説』p.45</a
          >
        </figcaption>
      </figure>
      <p>
        民営借家を、構造（木造・非木造）と延面積（30 m²未満・30
        m²以上）の組み合わせで4区分に分けます。
        家賃調査地区は市町村ごとに3群に分け、毎月1群ずつ調査します。当月に調査しない世帯には、直近の調査で得た家賃と延面積を使います。
      </p>
      <p>
        ここでは、ある調査市町村を仮定し、計算を簡単にするために木造小住宅と非木造中住宅の2区分で考えます。
      </p>
      <ol>
        <li>
          <p>
            区分ごとに、家賃の合計を延面積の合計で割り、3.3
            m²当たりの家賃を求めます。たとえば木造小住宅の2世帯は20 m²と25
            m²で、月額家賃が基準時には40,000円と50,000円、今月は44,000円と55,000円だったとします。
          </p>
          <MathFormula
            display
            tex={String.raw`\begin{aligned}
              P_0 &= \frac{40{,}000 + 50{,}000}{20 + 25} \times 3.3 = 6{,}600 \\
              P_t &= \frac{44{,}000 + 55{,}000}{20 + 25} \times 3.3 = 7{,}260
            \end{aligned}`}
          />
        </li>
        <li>
          <p>
            今月の平均価格を基準時の平均価格で割り、100を掛けます。木造小住宅の指数は110です。
          </p>
          <MathFormula
            display
            tex={String.raw`\frac{7{,}260}{6{,}600} \times 100 = 110`}
          />
          <p>
            非木造中住宅も同じ手順で計算します。2世帯の住宅の延面積を40 m²と60
            m²とし、月額家賃が基準時には90,000円と130,000円、今月は94,500円と136,500円だったとします。3.3
            m²当たりの平均家賃は次のとおりです。
          </p>
          <MathFormula
            display
            tex={String.raw`\begin{aligned}
              P_0 &= \frac{90{,}000 + 130{,}000}{40 + 60} \times 3.3 = 7{,}260 \\
              P_t &= \frac{94{,}500 + 136{,}500}{40 + 60} \times 3.3 = 7{,}623
            \end{aligned}`}
          />
          <p>
            今月の7,623円を基準時の7,260円で割り、100を掛けると、非木造中住宅の指数は105です。
          </p>
          <MathFormula
            display
            tex={String.raw`\frac{7{,}623}{7{,}260} \times 100 = 105`}
          />
        </li>
        <li>
          <p>
            区分ごとの指数を、家計調査の結果などに基づくウエイトで加重平均します（各区分のウエイトをどのように求めているのかはよくわかりませんでした）。
            この例では、ウエイトの比を木造小住宅、非木造中住宅の順に60:40とします。この市町村の民営家賃の指数は108なので、基準時から8％の上昇です。
          </p>
          <MathFormula
            display
            tex={String.raw`110 \times 0.6 + 105 \times 0.4 = 108`}
          />
        </li>
      </ol>
    </section>
  </article>
</main>

<style>
  .model-item-table {
    inline-size: 100%;
    table-layout: fixed;
    border-collapse: collapse;
    font-size: 0.95rem;
  }

  .model-item-table caption,
  .model-item-table th,
  .model-item-table td {
    border: var(--border-default);
    padding: 0.45rem 0.6rem;
    overflow-wrap: anywhere;
  }

  .model-item-table caption {
    border-block-end: 0;
    font-weight: 600;
  }

  .model-item-table thead th:first-child {
    inline-size: 24%;
  }

  .model-item-table thead th:last-child {
    inline-size: 18%;
  }

  .model-item-table td:last-child:not([colspan]) {
    text-align: center;
  }

  .model-item-table ol,
  .model-item-table ul {
    margin-block: 0.35rem;
    padding-inline-start: 1.5rem;
  }

  .model-item-table p {
    margin-block: 0.35rem;
  }

  blockquote {
    margin-inline: 0;
    padding-inline-start: 1rem;
    border-inline-start: var(--border-default);
  }

  .simulation {
    min-inline-size: 0;
    margin-inline: 0;
    border: var(--border-default);
    border-radius: var(--radius-md);
  }

  .weighted-index-chart :global(.chart-canvas) {
    height: 11rem;
  }

  .ingredient + .ingredient {
    margin-block-start: 1rem;
  }

  .simulation label {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 6ch;
    column-gap: 0.5rem;
    align-items: center;
  }

  .simulation label span:first-child {
    grid-column: 1 / -1;
  }

  .simulation label span:last-child {
    text-align: end;
  }

  .simulation input {
    width: 100%;
    margin: 0;
  }
</style>
