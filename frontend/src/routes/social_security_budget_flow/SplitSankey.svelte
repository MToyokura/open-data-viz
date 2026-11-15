<script>
  import {
    amountShare,
    formatApproximateAmount,
    formatExactAmount,
    formatExactYen,
    summarizeItems,
    totalValue,
  } from "./utils/amountComparison.js";
  import { buildSplitSankey } from "./utils/sankeyLayout.js";

  let { program } = $props();
  let income = $derived(summarizeItems(program.income));
  let expenditure = $derived(summarizeItems(program.expenditure));
  let layout = $derived(buildSplitSankey(program));
  let incomeTotal = $derived(totalValue(income));
  let expenditureTotal = $derived(totalValue(expenditure));
  /** @type {{ id: string, label: string, group: string, value: number } | null} */
  let activeItem = $state(null);
  let activeSide = $state("");
  let tooltipX = $state(8);
  let tooltipY = $state(8);
  let tooltipWidth = $state(320);
  let tooltipHeight = $state(120);
  const tooltipId = $props.id();
  /** @type {Record<string, string>} */
  const groupColors = {
    income: "#236b8e",
    transfer: "#a25c24",
    benefit: "#536c3f",
    other: "#705988",
  };

  /** @param {PointerEvent | FocusEvent | MouseEvent | KeyboardEvent} event @param {{ id: string, label: string, group: string, value: number }} item @param {string} side */
  function showItem(event, item, side) {
    activeItem = item;
    activeSide = side;
    positionTooltip(event);
  }

  /** @param {PointerEvent | FocusEvent | MouseEvent | KeyboardEvent} event */
  function positionTooltip(event) {
    const bounds = /** @type {Element} */ (
      event.currentTarget
    ).getBoundingClientRect();
    const pointer =
      "clientX" in event && (event.clientX !== 0 || event.clientY !== 0);
    const x = pointer ? event.clientX : bounds.left + bounds.width / 2;
    const y = pointer ? event.clientY : Math.max(0, bounds.top);
    tooltipX = Math.max(
      8,
      Math.min(x + 14, window.innerWidth - tooltipWidth - 8),
    );
    tooltipY = Math.max(
      8,
      y + 14 + tooltipHeight <= window.innerHeight - 8
        ? y + 14
        : y - tooltipHeight - 14,
    );
  }

  /** @param {KeyboardEvent} event @param {{ id: string, label: string, group: string, value: number }} item @param {string} side */
  function handleItemKeydown(event, item, side) {
    if (event.key === "Escape") hideItem();
    else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      showItem(event, item, side);
    }
  }

  function hideItem() {
    activeItem = null;
    activeSide = "";
  }

  /** @param {{ id: string, label: string, group: string, value: number }} item @param {string} side */
  function itemAriaLabel(item, side) {
    const total = side === "収入" ? incomeTotal : expenditureTotal;
    return `${item.label}、${formatExactAmount(item.value)}、${side}の表示項目計の${amountShare(item.value, total)}`;
  }
</script>

<svelte:window
  onscroll={hideItem}
  onresize={hideItem}
  onkeydown={(event) => {
    if (event.key === "Escape") hideItem();
  }}
/>

<section class="sankey" aria-label="収入と支出の分断サンキー図">
  <header class="summary">
    <div>
      <span>収入の表示項目計</span><strong
        >{formatApproximateAmount(incomeTotal)}</strong
      >
    </div>
    <div class="center-title">
      <span>2024年度</span><strong>{program.name}</strong><span
        >表示項目計の差（収入 − 支出）：{layout.totalDifference < 0
          ? "−"
          : ""}{formatApproximateAmount(Math.abs(layout.totalDifference))}</span
      >
    </div>
    <div>
      <span>支出の表示項目計</span><strong
        >{formatApproximateAmount(expenditureTotal)}</strong
      >
    </div>
  </header>
  <p class="scroll-hint">
    図の全体が表示されない場合は、左右にスクロールできます。
  </p>
  <div
    class="chart-scroll"
    role="region"
    aria-label="左右にスクロールできる財源・支出図"
  >
    <svg
      viewBox="0 0 {layout.width} {layout.height}"
      role="group"
      aria-label="左側に収入、右側に支出の項目を示す金額比例の帯。収入と支出は中央の空白で分かれています。"
    >
      <text class="side-title" x="8" y="30">収入</text><text
        class="side-title"
        x={layout.width - 8}
        y="30"
        text-anchor="end">支出</text
      >
      {#each layout.left.groupFrames as frame (frame.label)}<rect
          class="group-frame-fill"
          x="0"
          y={frame.top}
          width="195"
          height={frame.bottom - frame.top}
          rx="7"
        ></rect>{/each}
      {#each layout.right.groupFrames as frame (frame.label)}<rect
          class="group-frame-fill"
          x="805"
          y={frame.top}
          width="195"
          height={frame.bottom - frame.top}
          rx="7"
        ></rect>{/each}
      {#each layout.left.entries as item (item.id)}
        <g
          class={{
            highlighted: activeSide === "収入" && activeItem?.id === item.id,
          }}
          role="button"
          tabindex="0"
          aria-label={itemAriaLabel(item, "収入")}
          aria-describedby={activeSide === "収入" && activeItem?.id === item.id
            ? tooltipId
            : undefined}
          onclick={(event) => showItem(event, item, "収入")}
          onkeydown={(event) => handleItemKeydown(event, item, "収入")}
          onpointerenter={(event) => showItem(event, item, "収入")}
          onpointermove={(event) => showItem(event, item, "収入")}
          onpointerleave={hideItem}
          onfocus={(event) => showItem(event, item, "収入")}
          onblur={hideItem}
        >
          <path
            d={item.path}
            fill={groupColors[item.group] ?? "#536c3f"}
            fill-opacity="0.72"
          ></path>
          <rect
            x={layout.left.nodeX - 5}
            y={item.rowY - item.thickness / 2}
            width="10"
            height={Math.max(item.thickness, 0)}
            fill={groupColors[item.group] ?? "#536c3f"}
          ></rect>
          <line
            x1="160"
            x2={layout.left.nodeX - 6}
            y1={item.rowY}
            y2={item.rowY}
            stroke="#667781"
            stroke-width="1"
          ></line>
          <text class="item-label" x="8" y={item.rowY - 7}>{item.label}</text>
          <text class="item-amount" x="8" y={item.rowY + 15}
            >{formatApproximateAmount(item.value)} ・{amountShare(
              item.value,
              incomeTotal,
            )}</text
          >
        </g>
      {/each}
      {#each layout.right.entries as item (item.id)}
        <g
          class={{
            highlighted: activeSide === "支出" && activeItem?.id === item.id,
          }}
          role="button"
          tabindex="0"
          aria-label={itemAriaLabel(item, "支出")}
          aria-describedby={activeSide === "支出" && activeItem?.id === item.id
            ? tooltipId
            : undefined}
          onclick={(event) => showItem(event, item, "支出")}
          onkeydown={(event) => handleItemKeydown(event, item, "支出")}
          onpointerenter={(event) => showItem(event, item, "支出")}
          onpointermove={(event) => showItem(event, item, "支出")}
          onpointerleave={hideItem}
          onfocus={(event) => showItem(event, item, "支出")}
          onblur={hideItem}
        >
          <path
            d={item.path}
            fill={groupColors[item.group] ?? "#536c3f"}
            fill-opacity="0.72"
          ></path>
          <rect
            x={layout.right.nodeX - 5}
            y={item.rowY - item.thickness / 2}
            width="10"
            height={Math.max(item.thickness, 0)}
            fill={groupColors[item.group] ?? "#536c3f"}
          ></rect>
          <line
            x1={layout.right.nodeX + 6}
            x2="840"
            y1={item.rowY}
            y2={item.rowY}
            stroke="#667781"
            stroke-width="1"
          ></line>
          <text
            class="item-label"
            x={layout.width - 8}
            y={item.rowY - 7}
            text-anchor="end">{item.label}</text
          >
          <text
            class="item-amount"
            x={layout.width - 8}
            y={item.rowY + 15}
            text-anchor="end"
            >{formatApproximateAmount(item.value)} ・{amountShare(
              item.value,
              expenditureTotal,
            )}</text
          >
        </g>
      {/each}
      {#each layout.left.headers as heading (heading.label)}<text
          class="group-title"
          x="8"
          y={heading.y}>{heading.label}</text
        >{/each}
      {#each layout.right.headers as heading (heading.label)}<text
          class="group-title"
          x={layout.width - 8}
          y={heading.y}
          text-anchor="end">{heading.label}</text
        >{/each}
      {#each layout.left.groupFrames as frame (frame.label)}<path
          class="group-frame"
          d="M 0 {frame.top} H 195 M 0 {frame.top} V {frame.bottom} M 0 {frame.bottom} H 195"
        ></path>{/each}
      {#each layout.right.groupFrames as frame (frame.label)}<path
          class="group-frame"
          d="M 805 {frame.top} H 1000 M 1000 {frame.top} V {frame.bottom} M 805 {frame.bottom} H 1000"
        ></path>{/each}
      <line
        x1={layout.centerX - layout.centerGap / 2}
        x2={layout.centerX - layout.centerGap / 2}
        y1="50"
        y2={layout.height - 40}
        stroke="#9aa7ae"
        stroke-dasharray="4 5"
      ></line>
      <line
        x1={layout.centerX + layout.centerGap / 2}
        x2={layout.centerX + layout.centerGap / 2}
        y1="50"
        y2={layout.height - 40}
        stroke="#9aa7ae"
        stroke-dasharray="4 5"
      ></line>
    </svg>
  </div>
  <p class="explanation">
    帯の太さは金額に比例し、同じ制度の収入と支出には共通の尺度を使っています。制度を切り替えると帯幅の尺度が変わります。割合は、収入項目では収入の表示項目計、支出項目では支出の表示項目計に対する比率です。中央の空白は収入と支出の区切りで、左右の帯は個別の対応を示していません。割合の丸めにより合計が100%にならない場合があります。
  </p>

  {#if activeItem}
    <aside
      id={tooltipId}
      class="item-detail"
      role="tooltip"
      aria-live="polite"
      aria-label="項目詳細"
      style:left="{tooltipX}px"
      style:top="{tooltipY}px"
      bind:clientWidth={tooltipWidth}
      bind:clientHeight={tooltipHeight}
    >
      <div>
        <strong>{activeItem.label}</strong><strong
          >{formatExactYen(activeItem.value)}</strong
        ><span
          >{activeSide}の表示項目計に占める割合：{amountShare(
            activeItem.value,
            activeSide === "収入" ? incomeTotal : expenditureTotal,
          )}</span
        >
      </div>
      <p>項目コード：{activeItem.id}</p>
    </aside>
  {/if}
</section>

<style>
  .sankey {
    margin: var(--space-5) 0;
    min-width: 0;
  }
  .summary {
    display: grid;
    grid-template-columns: 1fr minmax(14rem, 2fr) 1fr;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    background: #f3f6f8;
    border-radius: var(--radius-md);
    text-align: center;
  }
  .summary div {
    display: grid;
    gap: var(--space-1);
    min-width: 0;
  }
  .summary span {
    color: #52616b;
    font-size: 0.88rem;
  }
  .summary strong {
    overflow-wrap: anywhere;
  }
  .center-title strong {
    font-size: 1.12rem;
  }
  .explanation {
    max-width: 100%;
    color: #46545d;
    font-size: 0.9rem;
    line-height: 1.7;
    overflow-wrap: anywhere;
  }
  .chart-scroll {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 1000px minmax(0, 1fr);
    width: 100%;
    overflow-x: auto;
  }
  .scroll-hint {
    margin: 0 0 var(--space-2);
    color: #52616b;
    font-size: 0.85rem;
  }
  svg {
    grid-column: 2;
    display: block;
    width: 1000px;
    max-width: none;
    height: auto;
    overflow: visible;
  }
  .side-title {
    fill: #263b49;
    font-size: 19px;
    font-weight: 700;
  }
  .item-label {
    fill: #17242c;
    font-size: 16px;
    font-weight: 600;
  }
  .item-amount {
    fill: #46545d;
    font-size: 13px;
  }
  .group-title {
    fill: #344650;
    font-size: 14px;
    font-weight: 700;
  }
  .group-frame-fill {
    fill: #f6f8f9;
  }
  .group-frame {
    fill: none;
    stroke: #c9d2d7;
    stroke-width: 1;
  }
  g[role="button"]:focus-visible {
    outline: 3px solid #e18d26;
    outline-offset: 3px;
  }
  g.highlighted path {
    fill-opacity: 1;
  }
  g.highlighted text {
    fill: #102a3a;
    font-weight: 700;
  }
  .item-detail {
    position: fixed;
    z-index: 1000;
    display: grid;
    gap: var(--space-1);
    max-width: min(320px, calc(100vw - 16px));
    box-sizing: border-box;
    padding: 0.75rem 1rem;
    border: 1px solid #cbd5e1;
    border-radius: var(--radius-md);
    background: #fff;
    color: #334155;
    font-size: 0.875rem;
    line-height: 1.6;
    box-shadow: 0 4px 16px rgb(51 65 85 / 14%);
    pointer-events: none;
  }
  .item-detail div {
    display: grid;
    gap: var(--space-1);
  }
  .item-detail p {
    margin: 0;
  }
  /* Sankey label arrangement within the chart, independent of page columns. */
  @media (max-width: 760px) {
    .summary {
      grid-template-columns: 1fr 1fr;
    }
    .center-title {
      grid-column: 1 / -1;
      grid-row: 1;
    }
  }
</style>
