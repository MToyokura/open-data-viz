# 消費者物価指数を理解する

このあたりを AI に放り込んで解説をしてもらうのが最も効果的な学び方かもしれません。

- [消費者物価指数に関するQ&A（回答）](https://www.stat.go.jp/data/cpi/4-1.html)
- [消費者物価指数のしくみと見方（PDF）](https://www.stat.go.jp/data/cpi/2025/mikata/pdf/0.pdf)
- [2025年基準 消費者物価指数の解説（PDF）](https://www.stat.go.jp/data/cpi/2025/kaisetsu/pdf/0.pdf)

一方で全体の概説のような資料を読むのもまた学びにつながるし私の学びにもなるのでその軌跡としてここに記します。

## 消費者物価指数とは

> 消費者物価指数は、全国の世帯が購入する財及びサービスの価格変動を総合的に測定し、物価の変動を時系列的に測定するものである。すなわち、消費者物価指数は、家計の消費構造を一定のものに固定し、これに要する費用が物価の変動によってどう変化するかを指数値で示したものである。

2025年（令和7年）基準 消費者物価指数の解説 p1 より。

指数は品目ごとのものと、まとめたものがあります。ニュースでよく見る「生鮮食品及びエネルギーを除く総合」は後者のまとめたものです。

（図）全国の生鮮食品及びエネルギーを除く総合（2025年＝100）

これが日本全体の「生鮮食品及びエネルギーを除く総合」の2025年基準消費者物価指数のグラフです。

## 「2025年基準」とは

> 消費者物価指数は、基準となる年の物価を100として、その時々の物価を比較計算した数値となっています。この、物価の基準となる時点のことを「指数の基準時」と呼んでいます。

> また、消費者物価指数は、ある時点の消費構造（品目ごとの支出割合）をウエイトとして、個々の品目の価格指数を加重平均して算出しています。この、ウエイトに採用した年次のことを、「ウエイトの参照年次」と呼んでいます。

> 現在、日本の消費者物価指数のうち毎月公表している公式系列については、「指数の基準時」と「ウエイトの参照年次」を共に2025年の1年間としており、これを「基準年」と呼んでいます。

> なお、基準年は、西暦年の末尾が0と5の年を基準時として、5年ごとに改定（基準改定）しています。その際、併せて指数に採用する品目などの見直しも行っています。

消費者物価指数に関するQ&A（回答）の A-3 より。

「2025年基準」とはざっくり言うと、2025年の1年間での指数を「100」とします、ということです。この1年間の指数がどう算出されるかは[後に解説](#基準時価格)します。

## 「ウエイト」とは

「ウエイト」とは家計の消費支出に占める割合であり、指数の計算において使用されます。

## 実際に指数を計算してみる

「消費者物価指数のしくみと見方」の p14 にあるカレーの例をそのまま使います。カレー全体が「消費支出」にあたり、各具材が具体的な品目にあたります。

> 米、牛肉及びカレールウの３品目によって物価指数を作成するとします。今月の価格が基準時に比べて、米が 20％値下がりして、基準時の 100 に対して 80 に、一方、牛肉は 20％値上がりして 120 に、カレールウも 15％値上がりして 115 になったとします。これを単純に平均すると、
>
> <math display="block"><mfrac><mrow><mn>80</mn><mo>+</mo><mn>120</mn><mo>+</mo><mn>115</mn></mrow><mn>3</mn></mfrac><mo>=</mo><mn>105</mn></math>
>
> となり、基準時の 100 に対して５％上昇したと計算されます。しかし、家計の消費支出上、この３品目に対する重要度は必ずしも同じではありません。この３品目の支出額の割合が、米６、牛肉３、カレールウ１であったとします。そこでこれらの値動きを、ウエイトを加味して計算すると、
>
> <math display="block"><mfrac><mrow><mn>80</mn><mo>×</mo><mn>6</mn><mo>+</mo><mn>120</mn><mo>×</mo><mn>3</mn><mo>+</mo><mn>115</mn><mo>×</mo><mn>1</mn></mrow><mrow><mn>6</mn><mo>+</mo><mn>3</mn><mo>+</mo><mn>1</mn></mrow></mfrac><mo>=</mo><mn>95.5</mn></math>
>
> となります。単純に計算した場合に比べてウエイトの大きさが反映され、4.5％の下落になりました。各品目の全体に占める支出額の割合を加味する、つまり、ウエイトを付けて平均する計算方法を「加重平均」といいます。

引用の「米6、牛肉3、カレールウ1」は、基準時の支出額の比であり、これがウエイトです。たとえば、基準時に米に600円、牛肉に300円、カレールウに100円を支出したとします。今月も同じ量を買うと、上の価格変化を反映した支出額は、米480円、牛肉360円、カレールウ115円です。合計は955円なので、基準時の1,000円を100とした指数は次のようになります。

<math display="block"><mfrac><mn>955</mn><mn>1,000</mn></mfrac><mo>×</mo><mn>100</mn><mo>=</mo><mn>95.5</mn></math>

このとき、今月の支出に占める米の割合は次のとおりです。

<math display="block"><mfrac><mn>480</mn><mn>955</mn></mfrac><mo>≈</mo><mn>50.3</mn><mo>%</mo></math>

米の値下がりとほかの品目の値上がりにより、米の支出割合は基準時の60％から今月の約50.3％に下がりましたが、この50.3％は、指数の計算には使いません。消費者物価指数の計算には基準時の支出比 6:3:1 を使うので、米のウエイトは6のままです。下記のシミュレーションでウエイトを変えると、基準時の支出比が異なるケースを試すことができます。

基準時からの価格変化とウエイトを変えて、加重平均を見てみましょう。

### 価格とウエイトのシミュレーション

基準時の合計を1,000円とし、今月も同じ量を買う場合の金額を表示します。

このシミュレーションでは、ウエイトの合計を10に固定しています。1品目のウエイトを変えると、残り2品目のウエイトも自動で調整されます。

初期設定の表示

| 品目       | 指数 | 基準時からの価格変化率 | 基準時のウエイト | 基準時 |  今月 |
| ---------- | ---: | ---------------------: | ---------------: | -----: | ----: |
| 米         |   80 |                  -20％ |                6 |  600円 | 480円 |
| 牛肉       |  120 |                  +20％ |                3 |  300円 | 360円 |
| カレールウ |  115 |                  +15％ |                1 |  100円 | 115円 |

合計 1,000円 → 955円

<math display="block"><mfrac><mn>955</mn><mn>1,000</mn></mfrac><mo>×</mo><mn>100</mn><mo>=</mo><mn>95.5</mn></math>

加重平均 95.5（基準時比 -4.5％）

各品目の指数に基準時の支出割合を掛け、その結果を足すと全体の指数になります。

<math display="block"><mrow><mn>0.6</mn><mo>×</mo><mn>80</mn><mo>+</mo><mn>0.3</mn><mo>×</mo><mn>120</mn><mo>+</mo><mn>0.1</mn><mo>×</mo><mn>115</mn><mo>=</mo><mn>95.5</mn></mrow></math>

（図）全体の指数に占める各品目の寄与

棒の各区間は、式の各項に対応します。

## ラスパイレス式

先ほどのカレーの例で、各品目の指数を基準時の支出比6、3、1で加重平均しました。実は、そのときに使った次の式がラスパイレス式です。

<math display="block"><mfrac><mrow><mn>80</mn><mo>×</mo><mn>6</mn><mo>+</mo><mn>120</mn><mo>×</mo><mn>3</mn><mo>+</mo><mn>115</mn><mo>×</mo><mn>1</mn></mrow><mrow><mn>6</mn><mo>+</mo><mn>3</mn><mo>+</mo><mn>1</mn></mrow></mfrac><mo>=</mo><mn>95.5</mn></math>

総務省統計局の『2025年基準 消費者物価指数の解説』（PDF）には、「指数算式」という用語が出てきます。これは、品目ごとの価格から全体の指数を求める計算式のことです。この資料では、消費者物価指数の計算に「基準時加重相対法算式（ラスパイレス型）」を用いるとしています。先ほどカレーの例で使ったラスパイレス式を一般化すると、次のように書けます。

<math display="block">
  <msub><mi>I</mi><mi>t</mi></msub><mo>=</mo>
  <mfrac><mrow><munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mi>i</mi></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></mrow><mrow><munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></mrow></mfrac><mo>×</mo><mn>100</mn><mo>=</mo>
  <mfrac><mrow><munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mi>i</mi></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></mfrac><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></mrow><mrow><munderover><mo>∑</mo><mrow><mi>i</mi><mo>=</mo><mn>1</mn></mrow><mi>n</mi></munderover><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></mrow></mfrac><mo>×</mo><mn>100</mn>
</math>

一般式の左側では、基準時と同じ量を買うのにかかる費用を、今月と基準時の価格で比べています。一般式の右側は、各品目の価格比を基準時の支出額で加重平均する形です。ウエイト <math><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub><mo>=</mo><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></math> を代入すると、2つの形が一致します。

- <math><msub><mi>I</mi><mi>t</mi></msub></math> は比較時の全体の指数で、この例では今月の95.5です。
- <math><mn>0</mn></math> は基準時、<math><mi>t</mi></math> は比較時（今月）です。
- <math><mi>i</mi></math> は品目を表し、1が米、2が牛肉、3がカレールウです。<math><mi>n</mi></math> は品目数で、この例では3です。<math><mo>∑</mo></math> は3品目分を足し合わせることを表します。
- <math><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></math> と <math><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mi>i</mi></mrow></msub></math> は、それぞれ基準時と今月の各品目の単価です。個々の単価は例に示されていませんが、価格比 <math><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mi>i</mi></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></mfrac></math> は米が0.80、牛肉が1.20、カレールウが1.15です。
- <math><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></math> は各品目の基準時の購入数量です。具体的な数量は例に示されていませんが、今月の費用を計算するときにも同じ数量を使います。
- <math><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mi>i</mi></mrow></msub></math> は基準時の支出額（単価×購入数量）です。米、牛肉、カレールウの順に600円、300円、100円で、比にすると6対3対1です。

まず、一般式の左側をカレーの例に当てはめます。基準時の購入数量を固定すると、今月の費用は、品目ごとに基準時の支出額に今月の価格比を掛けて求められます。たとえば米（品目1）は、基準時に600円分買った量を今月も買うと、単価が20％下がって基準時の0.80倍になっているため、費用も600円の0.80倍になります。米、牛肉、カレールウの順に、金額は次のとおりです（単位は円）。

<math display="block">
  <mtable>
    <mtr><mtd><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>1</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub></mtd><mtd><mo>=</mo></mtd><mtd><mn>0.80</mn><mo>×</mo><mn>600</mn></mtd><mtd><mo>=</mo></mtd><mtd><mn>480</mn></mtd></mtr>
    <mtr><mtd><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>2</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub></mtd><mtd><mo>=</mo></mtd><mtd><mn>1.20</mn><mo>×</mo><mn>300</mn></mtd><mtd><mo>=</mo></mtd><mtd><mn>360</mn></mtd></mtr>
    <mtr><mtd><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>3</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub></mtd><mtd><mo>=</mo></mtd><mtd><mn>1.15</mn><mo>×</mo><mn>100</mn></mtd><mtd><mo>=</mo></mtd><mtd><mn>115</mn></mtd></mtr>
  </mtable>
</math>

総和を3品目分に展開して金額を代入します。式の末尾の100と分母を約分すると、分母は6＋3＋1になります。

<math display="block">
  <mtable>
    <mtr><mtd><msub><mi>I</mi><mi>t</mi></msub></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mrow><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>1</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub><mo>+</mo><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>2</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub><mo>+</mo><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>3</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub></mrow><mrow><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub><mo>+</mo><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub><mo>+</mo><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub><msub><mi>q</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub></mrow></mfrac><mo>×</mo><mn>100</mn></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mrow><mn>480</mn><mo>+</mo><mn>360</mn><mo>+</mo><mn>115</mn></mrow><mrow><mn>600</mn><mo>+</mo><mn>300</mn><mo>+</mo><mn>100</mn></mrow></mfrac><mo>×</mo><mn>100</mn></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mrow><mn>480</mn><mo>+</mo><mn>360</mn><mo>+</mo><mn>115</mn></mrow><mrow><mn>6</mn><mo>+</mo><mn>3</mn><mo>+</mo><mn>1</mn></mrow></mfrac></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mn>955</mn><mn>10</mn></mfrac></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mn>95.5</mn></mtd></mtr>
  </mtable>
</math>

一般式の右側で使う「価格比」は、各品目の今月の単価を基準時の単価で割った値です。品目別の指数は基準時を100としているので、指数を100で割ると価格比になります。たとえば、米の指数80は、今月の単価が基準時の0.80倍であることを示しています。米、牛肉、カレールウの順に、価格比は次のとおりです。

<math display="block">
  <mtable>
    <mtr><mtd><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>1</mn></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub></mfrac></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mn>80</mn><mn>100</mn></mfrac></mtd><mtd><mo>=</mo></mtd><mtd><mn>0.80</mn></mtd></mtr>
    <mtr><mtd><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>2</mn></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub></mfrac></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mn>120</mn><mn>100</mn></mfrac></mtd><mtd><mo>=</mo></mtd><mtd><mn>1.20</mn></mtd></mtr>
    <mtr><mtd><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>3</mn></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub></mfrac></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mn>115</mn><mn>100</mn></mfrac></mtd><mtd><mo>=</mo></mtd><mtd><mn>1.15</mn></mtd></mtr>
  </mtable>
</math>

この価格比と基準時の支出額を右側の式に代入します。式の末尾の100と分母を約分すると、分母は6＋3＋1になります。分子の各項は、価格比を100倍して品目別指数にし、支出額を100で割って6、3、1にしても積は同じです。こうすると、先ほどの分数の式になります。さらに分子と分母を100で割ると、支出のウエイトは6、3、1になり、先ほどの分数の式に戻ります。

<math display="block">
  <mtable>
    <mtr><mtd><msub><mi>I</mi><mi>t</mi></msub></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mrow><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>1</mn></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub></mfrac><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub><mo>+</mo><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>2</mn></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub></mfrac><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub><mo>+</mo><mfrac><msub><mi>p</mi><mrow><mi>t</mi><mo>,</mo><mn>3</mn></mrow></msub><msub><mi>p</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub></mfrac><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub></mrow><mrow><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mn>1</mn></mrow></msub><mo>+</mo><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mn>2</mn></mrow></msub><mo>+</mo><msub><mi>w</mi><mrow><mn>0</mn><mo>,</mo><mn>3</mn></mrow></msub></mrow></mfrac><mo>×</mo><mn>100</mn></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mrow><mn>0.80</mn><mo>×</mo><mn>600</mn><mo>+</mo><mn>1.20</mn><mo>×</mo><mn>300</mn><mo>+</mo><mn>1.15</mn><mo>×</mo><mn>100</mn></mrow><mrow><mn>600</mn><mo>+</mo><mn>300</mn><mo>+</mo><mn>100</mn></mrow></mfrac><mo>×</mo><mn>100</mn></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mrow><mn>80</mn><mo>×</mo><mn>6</mn><mo>+</mo><mn>120</mn><mo>×</mo><mn>3</mn><mo>+</mo><mn>115</mn><mo>×</mo><mn>1</mn></mrow><mrow><mn>6</mn><mo>+</mo><mn>3</mn><mo>+</mo><mn>1</mn></mrow></mfrac></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mrow><mn>480</mn><mo>+</mo><mn>360</mn><mo>+</mo><mn>115</mn></mrow><mn>10</mn></mfrac></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mfrac><mn>955</mn><mn>10</mn></mfrac></mtd></mtr>
    <mtr><mtd></mtd><mtd><mo>=</mo></mtd><mtd><mn>95.5</mn></mtd></mtr>
  </mtable>
</math>

## 基準時価格

2025年の1年間の価格は、原則として基準年（2025年）の1月～12月の各月の比較時価格を単純平均して算出されます。（2025年基準 消費者物価指数の解説 p22 より）
