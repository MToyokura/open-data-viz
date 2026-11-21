<script>
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import ReferenceTable from "#lib/ReferenceTable.svelte";

  let { rebaseEnabled = false } = $props();

  let rows = $derived([
    { label: "統計名", value: "小売物価統計調査（動向編）" },
    {
      label: "表題",
      value:
        "主要品目の都市別小売価格－都道府県庁所在市及び人口15万以上の市(2000年1月～)",
    },
    {
      label: "統計表ID",
      id: "0003421913",
      url: "https://www.e-stat.go.jp/dbview?sid=0003421913",
      linkLabel: "e-Statで統計表を見る",
    },
    {
      label: "対象年／期間",
      value: "2000年1月～（収録期間は品目・都市によって異なります）",
    },
    {
      label: rebaseEnabled ? "指数" : "単位",
      value: rebaseEnabled
        ? "共通基準月＝100"
        : "円（価格に対応する数量・規格は品目ごとに異なります）",
    },
    {
      label: "調査品目及び基本銘柄",
      links: [
        {
          label: "調査品目及び基本銘柄",
          href: "https://www.stat.go.jp/data/kouri/doukou/3.html#meigara",
        },
      ],
    },
  ]);
</script>

<ReferencePanel title="データについて">
  <h3>利用上の注意</h3>
  <ul class="reference-notes">
    <li>
      品目の内容や基本銘柄は、総務省統計局の「調査品目及び基本銘柄」で確認できます。
    </li>
    <li>
      初期表示では、原データの価格（円）を表示します。「基準月で指数化」をオンにすると、各系列の基準月価格を100として「価格
      ÷ 基準月価格 × 100」で指数化します。
    </li>
  </ul>
  <h3>出典</h3>
  <ReferenceTable {rows} />
</ReferencePanel>
