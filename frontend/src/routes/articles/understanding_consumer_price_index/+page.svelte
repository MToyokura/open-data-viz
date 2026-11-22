<script lang="ts">
  import LoadState from "#lib/LoadState.svelte";
  import type { EChartsOption } from "echarts";
  import { onMount } from "svelte";
  import {
    fetchLookups,
    fetchSeries,
  } from "../../consumer_price_index/utils/fetchData.js";
  import { transformData } from "../../consumer_price_index/utils/transformData.js";
  import ArticleHeading from "../ArticleHeading.svelte";
  import CpiArticleChart from "../consumer_price_index/CpiArticleChart.svelte";

  const title = "消費者物価指数を理解する";
  const chartLabel = "全国の生鮮食品及びエネルギーを除く総合";
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
    if (!series?.timeValuePairs.some(([, value]) => value !== null))
      return null;

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

  const contributionChartLabel = "全体の指数に占める各品目の寄与";
  const contributionChart = $derived<EChartsOption>({
    animation: false,
    aria: { enabled: true },
    tooltip: {
      trigger: "axis",
      valueFormatter: (value) => `${value} ポイント`,
    },
    legend: { bottom: 8, selectedMode: false },
    grid: { left: 24, right: 24, top: 16, bottom: 64 },
    xAxis: {
      type: "value",
      min: 0,
      max: Math.max(100, Math.ceil(weightedIndex / 20) * 20),
      name: "指数（ポイント）",
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

  function setWeight(name: string, weight: number) {
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
</script>

<svelte:head>
  <title>{title} | Open Data Viz</title>
</svelte:head>

<main class="page-content">
  <article class="article-content">
    <header class="article-header">
      <ArticleHeading id="article-title" {title} level="h1" />
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
      一方で全体の概説のような資料を読むのもまた学びにつながるし私の学びにもなるのでその軌跡としてここに記します。
    </p>
    <section>
      <ArticleHeading id="cpi-definition" title="消費者物価指数とは" />
      <blockquote>
        <p>
          消費者物価指数は、全国の世帯が購入する財及びサービスの価格変動を総合的に測定し、物価の変動を時系列的に測定するものである。すなわち、消費者物価指数は、家計の消費構造を一定のものに固定し、これに要する費用が物価の変動によってどう変化するかを指数値で示したものである。
        </p>
      </blockquote>
      <p>2025年（令和7年）基準 消費者物価指数の解説 p1 より。</p>
      <p>
        指数は品目ごとのものと、まとめたものがあります。ニュースでよく見る「生鮮食品及びエネルギーを除く総合」は後者のまとめたものです。
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
      <p>
        これが日本全体の「生鮮食品及びエネルギーを除く総合」の2025年基準消費者物価指数のグラフです。
      </p>
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
      <p>消費者物価指数に関するQ&A（回答）の A-3 より。</p>
      <p>
        「2025年基準」とはざっくり言うと、2025年の1年間での指数を「100」とします、ということです。この1年間の指数がどう算出されるかは<a
          href="#基準時価格">後に解説</a
        >します。
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
        「消費者物価指数のしくみと見方」の p14
        にあるカレーの例をそのまま使います。カレー全体が「消費支出」にあたり、各具材が具体的な品目にあたります。
      </p>
      <blockquote>
        <p>
          米、牛肉及びカレールウの３品目によって物価指数を作成するとします。今月の価格が基準時に比べて、米が
          20％値下がりして、基準時の 100 に対して 80 に、一方、牛肉は
          20％値上がりして 120 に、カレールウも 15％値上がりして 115
          になったとします。これを単純に平均すると、
        </p>
        <math display="block"
          ><mfrac
            ><mrow><mn>80</mn><mo>+</mo><mn>120</mn><mo>+</mo><mn>115</mn></mrow
            ><mn>3</mn></mfrac
          ><mo>=</mo><mn>105</mn></math
        >
        <p>
          となり、基準時の 100
          に対して５％上昇したと計算されます。しかし、家計の消費支出上、この３品目に対する重要度は必ずしも同じではありません。この３品目の支出額の割合が、米６、牛肉３、カレールウ１であったとします。そこでこれらの値動きを、ウエイトを加味して計算すると、
        </p>
        <math display="block"
          ><mfrac
            ><mrow
              ><mn>80</mn><mo>×</mo><mn>6</mn><mo>+</mo><mn>120</mn><mo>×</mo
              ><mn>3</mn><mo>+</mo><mn>115</mn><mo>×</mo><mn>1</mn></mrow
            ><mrow><mn>6</mn><mo>+</mo><mn>3</mn><mo>+</mo><mn>1</mn></mrow
            ></mfrac
          ><mo>=</mo><mn>95.5</mn></math
        >
        <p>
          となります。単純に計算した場合に比べてウエイトの大きさが反映され、4.5％の下落になりました。各品目の全体に占める支出額の割合を加味する、つまり、ウエイトを付けて平均する計算方法を「加重平均」といいます。
        </p>
      </blockquote>
      <p>
        引用の「米6、牛肉3、カレールウ1」は、基準時の支出額の比であり、これがウエイトです。たとえば、基準時に米に600円、牛肉に300円、カレールウに100円を支出したとします。今月も同じ量を買うと、上の価格変化を反映した支出額は、米480円、牛肉360円、カレールウ115円です。合計は955円なので、基準時の1,000円を100とした指数は次のようになります。
      </p>
      <math display="block"
        ><mfrac><mn>955</mn><mn>1,000</mn></mfrac><mo>×</mo><mn>100</mn><mo
          >=</mo
        ><mn>95.5</mn></math
      >
      <p>このとき、今月の支出に占める米の割合は次のとおりです。</p>
      <math display="block"
        ><mfrac><mn>480</mn><mn>955</mn></mfrac><mo>≈</mo><mn>50.3</mn><mo>%</mo
        ></math
      >
      <p>
        消費者物価指数の計算には基準時の支出比 6:3:1
        を使うので、米のウエイトは6のままです。下記のシミュレーションでウエイトを変えると、基準時の支出比が異なるケースを試すことができます。
      </p>
      <p>基準時からの価格変化とウエイトを変えて、加重平均を見てみましょう。</p>
      <fieldset class="simulation">
        <legend>価格とウエイトのシミュレーション</legend>
        <p>
          基準時の合計を{formatAmount(
            baseTotal,
          )}とし、今月も同じ量を買う場合の金額を表示します。
        </p>
        <p>
          このシミュレーションでは、ウエイトの合計を{totalWeight}に固定しています。1品目のウエイトを変えると、残り2品目は変更前の比率に沿って整数で配分されます。残り2品目がともに0の場合は、できるだけ均等に配分します。
        </p>
        {#each ingredients as ingredient (ingredient.name)}
          {@const baseAmount = (baseTotal * ingredient.weight) / totalWeight}
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
                bind:value={
                  () => ingredient.weight,
                  (weight) => setWeight(ingredient.name, weight)
                }
              />
              <span>{ingredient.weight}</span>
            </label>
            <div>
              基準時 {formatAmount(baseAmount)} → 今月 {formatAmount(
                (baseAmount * (100 + ingredient.priceChange)) / 100,
              )}
            </div>
          </div>
        {/each}
        <p>
          <output>
            合計 {formatAmount(baseTotal)} → {formatAmount(currentTotal)}
            <math display="block" class="simulation-formula">
              <mfrac>
                <mn>{amountFormat.format(currentTotal)}</mn>
                <mn>{amountFormat.format(baseTotal)}</mn>
              </mfrac>
              <mo>×</mo>
              <mn>100</mn>
              <mo>{Number(displayedIndex) === weightedIndex ? "=" : "≈"}</mo>
              <mn>{displayedIndex}</mn>
            </math>
            加重平均 {displayedIndex}（基準時比 {formatChange(
              weightedIndex - 100,
            )}）
          </output>
        </p>
        <p>
          各品目の指数に基準時の支出割合を掛け、その結果を足すと全体の指数になります。
        </p>
        <math display="block">
          <mrow>
            {#each ingredients as ingredient, index (ingredient.name)}
              {#if index > 0}<mo>+</mo>{/if}
              <mn>{ingredient.weight / totalWeight}</mn>
              <mo>×</mo>
              <mn>{100 + ingredient.priceChange}</mn>
            {/each}
            <mo>=</mo>
            <mn>{displayedIndex}</mn>
          </mrow>
        </math>
        <figure class="article-chart contribution-chart">
          <CpiArticleChart
            chartData={contributionChart}
            label={contributionChartLabel}
          />
          <figcaption>棒の各区間は、式の各項に対応します。</figcaption>
        </figure>
      </fieldset>
    </section>
    <section>
      <ArticleHeading id="基準時価格" title="基準時価格" />
      <p>
        2025年の1年間の価格は、原則として基準年（2025年）の1月～12月の各月の比較時価格を単純平均して算出されます。（2025年基準
        消費者物価指数の解説 p22 より）
      </p>
    </section>
  </article>
</main>

<style>
  blockquote {
    margin-inline: 0;
    padding-inline-start: 1rem;
    border-inline-start: var(--border-default);
  }

  math {
    overflow-x: auto;
  }

  .simulation {
    min-inline-size: 0;
    margin-inline: 0;
    border: var(--border-default);
    border-radius: var(--radius-md);
  }

  .contribution-chart :global(.chart-canvas) {
    height: 9rem;
  }

  .simulation-formula {
    margin-block: 0.5rem;
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
