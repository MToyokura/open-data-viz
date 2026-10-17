<script>
  import { VITE_ROOT_PATH } from "#lib/paths.js";
</script>

<div class="page-shell">
  <main class="page-content">
    <h1>Open Data Viz</h1>
    <p>
      データの加工から可視化までのすべての工程を公開して、自由に利用できるようにすることをコンセプトに趣味で始めたプロジェクトです。
    </p>
    <p>ゆっくりですが内容を充実させていきたいと思っています。</p>
    <p>見た目もきれいにしていきたい...</p>
    <div>
      <p>今あるもの</p>
      <ul>
        <li>
          <a href={`${VITE_ROOT_PATH}/commodity_prices`}
            >主要品目の都市別小売価格</a
          >
        </li>
        <li>
          <a href={`${VITE_ROOT_PATH}/vegetable_production_relay`}
            >野菜の産地リレー</a
          >
        </li>
        <li>
          <a
            href={`${VITE_ROOT_PATH}/workers_by_age_employment_type_education_sex`}
            >年齢階級，教育，雇用形態別雇用者数</a
          >
        </li>
      </ul>
    </div>
    <div>
      <p>GitHub リポジトリ</p>
      <ul>
        <li>
          <a href="https://github.com/MToyokura/open-data-viz"
            >MToyokura/open-data-viz</a
          >
        </li>
      </ul>
    </div>
    <p>
      本サイトは <a
        href="https://creativecommons.org/publicdomain/zero/1.0/deed.ja"
        >CC0 1.0</a
      > で公開しています。
    </p>
  </main>
</div>
