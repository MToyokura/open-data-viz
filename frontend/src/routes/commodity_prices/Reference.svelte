<script>
  import ReferencePanel from "#lib/ReferencePanel.svelte";
</script>

<ReferencePanel title="データについて">
  <h3>利用上の注意</h3>
  <ul class="reference-notes">
    <li>
      品目の内容や基本銘柄は、総務省統計局の「調査品目及び基本銘柄」で確認できます。
    </li>
  </ul>
  <h3>出典</h3>
  <table class="reference-table ui-table">
    <tbody>
      <tr>
        <th>統計名</th>
        <td>小売物価統計調査（動向編）</td>
      </tr>
      <tr>
        <th>表題</th>
        <td>
          主要品目の都市別小売価格－都道府県庁所在市及び人口15万以上の市(2000年1月～)
        </td>
      </tr>
      <tr>
        <th>統計表ID</th>
        <td>
          <div>0003421913</div>
          <div>
            <a
              href="https://www.e-stat.go.jp/dbview?sid=0003421913"
              target="_blank"
              rel="noopener noreferrer">e-Statで統計表を見る</a
            >
          </div>
        </td>
      </tr>
      <tr>
        <th>対象年／期間</th>
        <td>2000年1月～（収録期間は品目・都市によって異なります）</td>
      </tr>
      <tr>
        <th>単位</th>
        <td>円（価格に対応する数量・規格は品目ごとに異なります）</td>
      </tr>
      <tr>
        <th>調査品目及び基本銘柄</th>
        <td>
          <div>
            <a
              href="https://www.stat.go.jp/data/kouri/doukou/3.html#meigara"
              target="_blank"
              rel="noopener noreferrer">調査品目及び基本銘柄</a
            >
          </div>
        </td>
      </tr>
    </tbody>
  </table>
</ReferencePanel>
