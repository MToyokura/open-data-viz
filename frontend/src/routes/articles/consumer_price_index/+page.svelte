<script lang="ts">
  import LoadState from "#lib/LoadState.svelte";
  import { VITE_ROOT_PATH } from "#lib/paths.js";
  import { onMount } from "svelte";
  import {
    fetchLookups,
    fetchSeries,
  } from "../../consumer_price_index/utils/fetchData.js";
  import { transformData } from "../../consumer_price_index/utils/transformData.js";
  import ArticleHeading from "../ArticleHeading.svelte";
  import CpiArticleChart from "./CpiArticleChart.svelte";

  const charts = {
    "0001": { itemCodes: ["0001"], startMonth: "" },
    "0002": { itemCodes: ["0002"] },
    "1461": { itemCodes: ["1461"] },
    "1761": { itemCodes: ["1761"] },
    "1914": { itemCodes: ["1914"] },
    "3000": { itemCodes: ["3000"], startMonth: "" },
    "3500": { itemCodes: ["3500"] },
    appliances: { itemCodes: ["4003", "4021", "4042"] },
    "4063": { itemCodes: ["4063"] },
    "4413": { itemCodes: ["4413"] },
    "0120": { itemCodes: ["0120"] },
    "0137": { itemCodes: ["0137"] },
    "9372": { itemCodes: ["9372"] },
    "0162": { itemCodes: ["0162"] },
    toiletries: {
      itemCodes: ["9621", "9626", "9622"],
      referenceMonth: "2005-01",
    },
  } satisfies Record<string, ChartDefinition>;
  const areaCode = "00000";
  const defaultReferenceMonth = "2000-01";
  type ChartDefinition = {
    itemCodes: string[];
    referenceMonth?: string;
    startMonth?: string;
  };
  type ChartKey = keyof typeof charts;
  type LookupEntry = { code: string; name: string };

  let chartRequests = $state<
    Partial<Record<ChartKey, ReturnType<typeof loadChart>>>
  >({});

  function getReferenceMonth(chart: ChartDefinition) {
    return chart.referenceMonth ?? defaultReferenceMonth;
  }

  async function loadChart(
    chartKey: ChartKey,
    lookups: ReturnType<typeof fetchLookups>,
  ) {
    const chart: ChartDefinition = charts[chartKey];
    const { items, regions }: { items: LookupEntry[]; regions: LookupEntry[] } =
      await lookups;
    const region = regions.find((region) => region.code === areaCode);
    if (!region) return null;

    const series = await Promise.all(
      chart.itemCodes.map(async (itemCode) => {
        const itemIndex = items.findIndex((item) => item.code === itemCode);
        if (itemIndex === -1) return null;

        const itemSeries = await fetchSeries({
          itemCode,
          areaCode,
          itemIndex,
          itemCount: items.length,
        });
        if (!itemSeries) return null;

        return {
          ...itemSeries,
          itemName: items[itemIndex].name,
          areaName: region.name,
        };
      }),
    );

    if (!series.every((item) => item !== null)) return null;
    return transformData(
      series,
      "normalized",
      getReferenceMonth(chart),
      chart.startMonth ?? "2000-01",
    ).chartData;
  }

  onMount(() => {
    const lookups = fetchLookups();
    for (const chartKey of Object.keys(charts) as ChartKey[]) {
      chartRequests[chartKey] = loadChart(chartKey, lookups);
    }
  });
</script>

<svelte:head>
  <title>消費者物価指数の推移を見る | Open Data Viz</title>
</svelte:head>

{#snippet cpiChart(chartKey: ChartKey, itemName: string)}
  {@const label = `全国の消費者物価指数（${itemName}）`}
  {@const [year, month] = getReferenceMonth(charts[chartKey]).split("-")}
  <figure class="article-chart">
    {#await chartRequests[chartKey]}
      <LoadState state="loading" message="データを読み込んでいます…" />
    {:then chartData}
      {#if chartData}
        <CpiArticleChart {chartData} {label} />
      {:else if chartRequests[chartKey]}
        <p role="status">表示できるデータがありません。</p>
      {:else}
        <LoadState state="loading" message="データを読み込んでいます…" />
      {/if}
    {:catch}
      <LoadState
        state="error"
        message="データを取得できませんでした。"
        onretry={() =>
          (chartRequests[chartKey] = loadChart(chartKey, fetchLookups()))}
      />
    {/await}
    <figcaption>{label}（{year}年{Number(month)}月＝100）</figcaption>
  </figure>
{/snippet}

<main class="page-content">
  <article class="article-content">
    <header class="article-header">
      <ArticleHeading
        id="article-title"
        title="消費者物価指数の推移を見る"
        level="h1"
      />
    </header>

    <p>
      消費者物価指数の品目・地域の組み合わせのうち、個人的に興味深く感じたものをいくつかピックアップしたいと思います。
    </p>
    <p>ご自身で探索したい場合はこちらへ。</p>
    <a class="card" href={`${VITE_ROOT_PATH}/consumer_price_index`}
      >消費者物価指数</a
    >
    <p>
      本記事では総務省の「<a
        href="https://www.e-stat.go.jp/dbview?sid=0004052037"
        >2025年基準 消費者物価指数</a
      >」を使用しています。長期的な変化を見やすくするため図では2000年1月を基準月（=
      100）として再計算しています。また、特記がない限り、グラフは2000年1月から表示しています。データはすべて「全国」の値を用いています。
    </p>
    <section>
      <ArticleHeading id="all-items" title="総合" />
      <p>
        興味深いかと言うとそうでもないですが、まずは全体を把握すべきであろうということで総合の推移を見てみます。
      </p>
      <p>こちらのグラフは全期間を表示対象としています。</p>
      <p>1993年から2021年まで横ばいでしたが、2022年から上昇しています。</p>

      {@render cpiChart("0001", "総合")}
    </section>

    <section>
      <ArticleHeading id="food" title="食料" />
      <p>2014年から上昇傾向にあるようです。</p>

      {@render cpiChart("0002", "食料")}
    </section>

    <section>
      <ArticleHeading id="dried-nori" title="干しのり" />
      <p>2023年頃から急上昇中みたいです。</p>

      {@render cpiChart("1461", "干しのり")}
    </section>

    <section>
      <ArticleHeading id="chocolate" title="チョコレート" />
      <p>2023年から上昇しています。</p>

      {@render cpiChart("1761", "チョコレート")}

      <p>
        <a
          href="https://www.icco.org/august-2024-quarterly-bulletin-of-cocoa-statistics/"
          >2023年の不作</a
        >による供給減少が指数上昇の一因と考えられます。
      </p>

      <figure class="article-chart">
        <iframe
          title="地域別のカカオ豆生産量"
          src="https://ourworldindata.org/grapher/cocoa-beans-production-by-region?time=2000..latest&amp;tab=chart"
          loading="lazy"
          style="width: 100%; height: 400px; border: 0px none;"
          allow="web-share; clipboard-write"
        ></iframe>
        <figcaption>地域別のカカオ豆生産量 Our World in Data</figcaption>
      </figure>
    </section>

    <section>
      <ArticleHeading id="tea-drinks" title="茶飲料" />
      <p>減少傾向です。</p>

      {@render cpiChart("1914", "茶飲料")}
    </section>

    <section>
      <ArticleHeading id="private-rent" title="民営家賃" />
      <p>長期推移が興味深いので1970年からのデータを表示しています。</p>
      <p>1998年からずっと横ばいです。</p>

      {@render cpiChart("3000", "民営家賃")}
    </section>

    <section>
      <ArticleHeading id="electricity" title="電気代" />
      <p>2024年以降は周期性が見られます。</p>

      {@render cpiChart("3500", "電気代")}
    </section>

    <section>
      <ArticleHeading id="home-appliances" title="家電" />
      <p>電子レンジ、冷蔵庫、全自動洗濯機はいずれも大きく下がっています。</p>

      {@render cpiChart("appliances", "電子レンジ・冷蔵庫・全自動洗濯機")}

      <p>ルームエアコンは、長らく下がっていましたが、近年は上昇しています。</p>

      <p>1月に指数が下がる傾向が見られます。</p>

      {@render cpiChart("4063", "ルームエアコン")}
    </section>

    <section>
      <ArticleHeading id="toilet-paper" title="トイレットペーパー" />
      <p>上がっています。</p>

      {@render cpiChart("4413", "トイレットペーパー")}
    </section>

    <section>
      <ArticleHeading id="textbooks" title="教科書・学習参考教材" />
      <p>上がっています。</p>

      {@render cpiChart("0120", "教科書・学習参考教材")}
    </section>

    <section>
      <ArticleHeading id="education-costs" title="教育関係費" />
      <p>やや下がっています。</p>

      {@render cpiChart("0162", "教育関係費")}
    </section>

    <section>
      <ArticleHeading id="books" title="書籍" />
      <p>上がっています。</p>

      {@render cpiChart("0137", "書籍")}
    </section>

    <section>
      <ArticleHeading id="theme-park-admission" title="テーマパーク入場料" />
      <p>上がっています。</p>

      <p>
        2023年1月の急落が気になったので ChatGPT
        に聞いたところ、2022年10月から2023年1月末まで<a
          href="https://www.meti.go.jp/covid-19/event-wakuwaku/index.html"
          >「イベント割事業」という政策</a
        >があったらしく、それが関係している可能性がありそうとのことです。
      </p>

      {@render cpiChart("9372", "テーマパーク入場料")}
    </section>

    <section>
      <ArticleHeading
        id="soap-and-shampoo"
        title="手洗い用石けん・ボディーソープ・シャンプー"
      />
      <p>シャンプーだけ上がっています。</p>

      {@render cpiChart(
        "toiletries",
        "手洗い用石けん・ボディーソープ・シャンプー",
      )}
    </section>

    <section>
      <ArticleHeading id="summary" title="まとめ" />
      <p>感覚と一致していたり、違ったりして面白いですね。</p>
      <p>
        なお、消費者物価指数は単に数値であり、本当に重要なのはその数値の解釈です。上昇・下降した要因、それによって消費者が受けた影響などについて結論を導くことはできませんので注意する必要があります。
      </p>
    </section>
  </article>
</main>
