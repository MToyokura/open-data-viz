<script lang="ts">
  import LicenseLink from "#lib/LicenseLink.svelte";
  import githubIcon from "#lib/assets/icons/GitHub_Invertocat_Black.svg";
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  const categories = [
    {
      id: "prices",
      title: "物価関連",
      projects: [
        {
          title: "消費者物価指数",
          href: `${VITE_ROOT_PATH}/consumer_price_index`,
        },
        {
          title: "主要品目の都市別小売価格",
          href: `${VITE_ROOT_PATH}/commodity_prices`,
        },
      ],
    },
    {
      id: "food",
      title: "農林水産",
      projects: [
        {
          title: "野菜の産地リレー",
          href: `${VITE_ROOT_PATH}/vegetable_production_relay`,
        },
        {
          title: "海面漁業魚種別漁獲量",
          href: `${VITE_ROOT_PATH}/marine_fishery_catch`,
        },
      ],
    },
    {
      id: "society",
      title: "雇用・資産・社会保障",
      projects: [
        {
          title: "年齢階級，教育，雇用形態別雇用者数",
          href: `${VITE_ROOT_PATH}/workers_by_age_employment_type_education_sex`,
        },
        {
          title: "日本の純資産分布",
          href: `${VITE_ROOT_PATH}/asset_distribution`,
        },
        {
          title: "社会保障給付費収支",
          href: `${VITE_ROOT_PATH}/social_security_budget_flow`,
        },
      ],
    },
  ];

  const articles = [
    {
      title: "消費者物価指数の推移を見る",
      href: `${VITE_ROOT_PATH}/articles/consumer_price_index`,
    },
    {
      title: "消費者物価指数を理解する",
      href: `${VITE_ROOT_PATH}/articles/understanding_consumer_price_index`,
    },
    {
      title: "モノの価格の推移を見る",
      href: `${VITE_ROOT_PATH}/articles/commodity_prices`,
    },
  ];
</script>

{#snippet linkList(links: { title: string; href: string }[])}
  <ul role="list">
    {#each links as link (link.href)}
      <li>
        <a class="row" href={link.href}>
          <strong>{link.title}</strong>
          <span class="arrow" aria-hidden="true">↗</span>
        </a>
      </li>
    {/each}
  </ul>
{/snippet}

<div class="home" lang="ja">
  <header class="mast wrap">
    <a class="wordmark" href={`${VITE_ROOT_PATH}/`}>Open Data Viz</a>
  </header>

  <main>
    <section class="hero wrap">
      <h1>Open Data Viz</h1>
      <p class="intro">
        データの加工から可視化までのすべての工程を公開して、<br
          class="intro-break"
        />自由に利用できるようにすることをコンセプトに趣味で始めたプロジェクトです。
      </p>
      <p class="intro">
        ゆっくりですが内容を充実させていきたいと思っています。
      </p>
      <p class="intro">
        本サイトのコンテンツは特記がない限り
        <a
          href="https://creativecommons.org/publicdomain/zero/1.0/deed.ja"
          target="_blank"
          rel="noreferrer">CC0</a
        >
        で公開（著作権を最大限の範囲で放棄）<br
          class="intro-break"
        />していますのでご自由にお使いください。
      </p>
    </section>

    <div class="wrap directory">
      <section class="projects" id="projects" aria-labelledby="projects-title">
        <h2 id="projects-title">テーマ一覧</h2>
        {#each categories as category (category.id)}
          <section class="group" aria-labelledby={category.id}>
            <h3 id={category.id}>{category.title}</h3>
            {@render linkList(category.projects)}
          </section>
        {/each}
      </section>

      <section
        class="link-section"
        id="articles"
        aria-labelledby="reading-title"
      >
        <h2 id="reading-title">物価の関連記事</h2>
        {@render linkList(articles)}
      </section>

      <section class="link-section" aria-labelledby="source-title">
        <h2 id="source-title">ソースコード</h2>
        <a
          class="row github-link"
          href="https://github.com/MToyokura/open-data-viz"
        >
          <strong>
            <span
              class="github-icon"
              style:--github-icon={`url("${githubIcon}")`}
              aria-hidden="true"
            ></span>
            GitHub リポジトリ
          </strong>
          <span class="arrow" aria-hidden="true">↗</span>
        </a>
      </section>
    </div>
  </main>

  <footer class="site-footer">
    <LicenseLink variant="footer" />
  </footer>
</div>
