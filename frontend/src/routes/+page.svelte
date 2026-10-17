<script>
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  const projects = [
    {
      title: "主要品目の都市別小売価格",
      href: `${VITE_ROOT_PATH}/commodity_prices`,
    },
    {
      title: "野菜の産地リレー",
      href: `${VITE_ROOT_PATH}/vegetable_production_relay`,
    },
    {
      title: "年齢階級，教育，雇用形態別雇用者数",
      href: `${VITE_ROOT_PATH}/workers_by_age_employment_type_education_sex`,
    },
    {
      title: "日本の純資産分布",
      href: `${VITE_ROOT_PATH}/asset_distribution`,
    },
  ];
</script>

<div class="page-shell">
  <main class="page-content">
    <h1>Open Data Viz</h1>
    <p>
      データの加工から可視化までのすべての工程を公開して、自由に利用できるようにすることをコンセプトに趣味で始めたプロジェクトです。
    </p>
    <p>ゆっくりですが内容を充実させていきたいと思っています。</p>

    <section aria-labelledby="projects-title">
      <h2 id="projects-title">今あるもの</h2>
      <div class="card-grid">
        {#each projects as project (project.href)}
          <a class="card" href={project.href}>{project.title}</a>
        {/each}
      </div>
    </section>

    <div>
      <h2>GitHub リポジトリ</h2>
      <ul>
        <li>
          <a href="https://github.com/MToyokura/open-data-viz"
            >MToyokura/open-data-viz</a
          >
        </li>
      </ul>
    </div>

    <section aria-labelledby="license-title">
      <h2 id="license-title">ライセンス</h2>
      <p>
        本サイトは <a
          href="https://creativecommons.org/publicdomain/zero/1.0/deed.ja"
          >CC0 1.0</a
        > で公開しています。
      </p>
    </section>
  </main>
</div>

<style>
  h1 {
    font-size: 2rem;
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 16rem), 1fr));
    gap: 1rem;
  }

  .card {
    display: block;
    padding: 1rem;
    border: var(--border-default);
    border-radius: var(--radius-md);
    color: inherit;
    text-decoration: none;
  }
</style>
