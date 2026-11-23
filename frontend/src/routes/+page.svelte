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

  <footer>
    <div class="footer-content">
      <LicenseLink variant="footer" />
    </div>
  </footer>
</div>

<style>
  .home {
    --paper: #f1f5fc;
    --ink: #142343;
    --muted: #425372;
    --line: #c9d5e9;
    --accent: #244dc2;

    display: flex;
    flex-grow: 1;
    flex-direction: column;
    background: var(--paper);
    color: var(--ink);
    color-scheme: light;
    font:
      1rem/1.65 system-ui,
      "Hiragino Sans",
      "Yu Gothic",
      sans-serif;
  }
  .wrap {
    box-sizing: border-box;
    width: min(var(--page-content-max-width), calc(100% - 3rem));
    margin-inline: auto;
  }
  h1,
  h2,
  h3,
  p,
  ul {
    margin: 0;
  }
  ul {
    padding: 0;
    list-style: none;
  }
  a {
    color: inherit;
    text-underline-offset: 0.23em;
  }
  a:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
  }
  .mast {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    padding-block: 1.25rem;
  }
  .wordmark {
    font-weight: 750;
    text-decoration: none;
  }
  .row:hover strong {
    text-decoration: underline;
  }
  h1 {
    line-height: 1.05;
    letter-spacing: -0.065em;
    text-wrap: balance;
  }
  h2 {
    font-size: 1.3rem;
  }
  h3 {
    font-size: 0.95rem;
  }
  .intro {
    color: var(--muted);
    margin-top: 0.55rem;
  }
  .row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.8rem;
    text-decoration: none;
  }
  .row strong {
    font-weight: 550;
  }
  .arrow {
    color: var(--accent);
    flex: none;
  }
  footer {
    margin-top: auto;
  }
  .footer-content {
    display: flex;
    justify-content: end;
    padding: var(--space-4);
  }

  .github-link {
    width: fit-content;
    margin-top: 0.75rem;
  }

  .github-link strong {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .github-icon {
    flex: none;
    width: 1rem;
    height: 1rem;
    background-color: currentColor;
    mask: var(--github-icon) center / contain no-repeat;
  }
  @media (max-width: 480px) {
    .wrap {
      width: calc(100% - 2rem);
    }
    .mast {
      gap: 0.55rem;
    }
  }

  .hero {
    background: #dce7ff;
    padding: clamp(1.4rem, 5vw, 4rem);
    border-radius: 1.25rem;
    margin-block: 0.5rem 2.5rem;
  }
  .hero h1 {
    font-size: clamp(2.85rem, 9vw, 7rem);
    font-weight: 830;
    color: #193780;
  }
  .intro {
    max-width: 40rem;
  }
  .hero h1 + .intro {
    margin-top: 2rem;
  }
  .intro + .intro {
    margin-top: 0.85rem;
  }
  @media (max-width: 48rem) {
    .intro-break {
      display: none;
    }
  }
  .directory {
    padding-bottom: 2.75rem;
  }
  .projects {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1rem;
  }
  .projects h2 {
    grid-column: 1 / -1;
  }
  .group {
    padding: 1rem;
    border-radius: 1rem;
  }
  .group h3 {
    color: var(--accent);
    padding-bottom: 0.55rem;
  }
  .row {
    padding: 0.65rem 0.6rem;
    border-radius: 0.45rem;
  }
  .group li + li {
    border-top: 1px solid var(--line);
  }
  .link-section {
    margin-top: 2.1rem;
  }
  .link-section ul {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }
  .link-section .row {
    align-items: center;
    padding: 0.75rem 1rem;
  }
  .group,
  .link-section .row {
    background: white;
    border: 1px solid var(--accent);
  }
  .row:hover {
    background: #eaf0ff;
  }
  .group .row:hover {
    border-radius: 0;
  }
  @media (max-width: 760px) {
    .projects {
      grid-template-columns: 1fr;
    }
  }
</style>
