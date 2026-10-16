<script>
  import { format } from "d3";
  import { sankey, sankeyLinkHorizontal } from "d3-sankey";
  /** @typedef {import("./constants/types.js").RelaySankeyData} RelaySankeyData */
  /** @typedef {import("./constants/types.js").RelaySankeyNode} RelaySankeyNode */
  /** @typedef {import("./constants/types.js").RelaySankeyLink} RelaySankeyLink */
  /** @typedef {import("d3-sankey").SankeyNode<RelaySankeyNode, {value: number}>} SankeyNode */
  /** @type {{data: RelaySankeyData}} */
  let { data } = $props();
  let containerWidth = $state(800);
  let width = $derived(Math.max(640, containerWidth));
  const height = 1600;
  const quantity = format(",.6~f");
  /** @type {{label: string, x: number, y: number, data: RelaySankeyData} | undefined} */
  let tooltip = $state.raw();
  let tooltipWidth = $state(320);
  let tooltipHeight = $state(90);
  const tooltipId = $props.id();
  let visibleTooltip = $derived(tooltip?.data === data ? tooltip : undefined);

  /** @param {PointerEvent | MouseEvent | FocusEvent | KeyboardEvent} event @param {string} label */
  function showTooltip(event, label) {
    const bounds = /** @type {SVGElement} */ (
      event.currentTarget
    ).getBoundingClientRect();
    const pointer =
      "clientX" in event && (event.clientX !== 0 || event.clientY !== 0);
    const x = pointer ? event.clientX : bounds.left + bounds.width / 2;
    const y = pointer ? event.clientY : Math.max(0, bounds.top);
    tooltip = {
      label,
      data,
      x: Math.max(8, Math.min(x + 14, window.innerWidth - tooltipWidth - 8)),
      y: Math.max(
        8,
        y + 14 + tooltipHeight <= window.innerHeight - 8
          ? y + 14
          : y - tooltipHeight - 14,
      ),
    };
  }
  function hideTooltip() {
    tooltip = undefined;
  }
  /** @param {KeyboardEvent} event @param {string} label */
  function tooltipKeydown(event, label) {
    if (event.key === "Escape") hideTooltip();
    else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      showTooltip(event, label);
    }
  }
  let graph = $derived.by(() => {
    if (!data.links.length) return undefined;
    const layout =
      /** @type {import("d3-sankey").SankeyLayout<{nodes: RelaySankeyNode[], links: RelaySankeyLink[]}, RelaySankeyNode, {value: number}>} */ (
        /** @type {unknown} */ (sankey())
      );
    return layout
      .nodeId((node) => node.id)
      .nodeAlign((node) => (node.side === "source" ? 0 : 1))
      .nodeSort((a, b) => a.order - b.order)
      .nodeWidth(16)
      .nodePadding(24)
      .extent([
        [110, 30],
        [width - 110, height - 30],
      ])({
      nodes: data.nodes.map((node) => ({ ...node })),
      links: data.links.map((link) => ({ ...link })),
    });
  });
  const path = sankeyLinkHorizontal();
  /** @param {SankeyNode} node */
  function nodeLabel(node) {
    return `${node.side === "source" ? "産地" : "消費地域"}：${node.name}、産地都道府県別の卸売数量 ${quantity(node.value ?? 0)} t`;
  }
  /** @param {unknown} node */
  function asSankeyNode(node) {
    return /** @type {SankeyNode} */ (node);
  }
</script>

<svelte:window
  onscroll={hideTooltip}
  onresize={hideTooltip}
  onkeydown={(event) => {
    if (event.key === "Escape") hideTooltip();
  }}
/>

{#if visibleTooltip}
  <div
    id={tooltipId}
    class="quantity-tooltip"
    role="tooltip"
    style:left={`${visibleTooltip.x}px`}
    style:top={`${visibleTooltip.y}px`}
    bind:clientWidth={tooltipWidth}
    bind:clientHeight={tooltipHeight}
  >
    {visibleTooltip.label}
  </div>
{/if}

<div class="chart-container" bind:clientWidth={containerWidth}>
  <p class="scroll-note">
    線やノードにポインターを合わせるか、タップ・Tab キーで数量を確認できます。
  </p>
  {#if graph}
    <!-- Keyboard users need to scroll the chart region. -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="chart-scroll"
      onscroll={hideTooltip}
      tabindex="0"
      role="region"
      aria-label="産地都道府県別の卸売数量の Sankey 図。横スクロール可能"
    >
      <svg
        {width}
        {height}
        viewBox={`0 0 ${width} ${height}`}
        role="group"
        aria-label="左に産地都道府県47区分、右に主要消費地域9区分。線の幅は卸売数量に比例します。"
      >
        <text x="110" y="18">産地都道府県</text>
        <text x={width - 110} y="18" text-anchor="end">主要消費地域</text>
        {#each graph.links as link (`${asSankeyNode(link.source).id}:${asSankeyNode(link.target).id}`)}
          {const source = $derived(asSankeyNode(link.source))}
          {const target = $derived(asSankeyNode(link.target))}
          {const label = $derived(
            `産地：${source.name} → 消費地域：${target.name}、産地都道府県別の卸売数量 ${quantity(link.value)} t`,
          )}
          <path
            d={path(link) ?? ""}
            stroke={source.color}
            stroke-width={link.width}
            fill="none"
            tabindex="0"
            role="button"
            aria-label={label}
            aria-describedby={visibleTooltip?.label === label
              ? tooltipId
              : undefined}
            onclick={(event) => showTooltip(event, label)}
            onkeydown={(event) => tooltipKeydown(event, label)}
            onpointerenter={(event) => showTooltip(event, label)}
            onpointermove={(event) => showTooltip(event, label)}
            onpointerleave={hideTooltip}
            onfocus={(event) => showTooltip(event, label)}
            onblur={hideTooltip}
          >
          </path>
        {/each}
        {#each graph.nodes as node (node.id)}
          {@const label = nodeLabel(node)}
          <g
            tabindex="0"
            role="button"
            aria-label={label}
            aria-describedby={visibleTooltip?.label === label
              ? tooltipId
              : undefined}
            onclick={(event) => showTooltip(event, label)}
            onkeydown={(event) => tooltipKeydown(event, label)}
            onpointerenter={(event) => showTooltip(event, label)}
            onpointermove={(event) => showTooltip(event, label)}
            onpointerleave={hideTooltip}
            onfocus={(event) => showTooltip(event, label)}
            onblur={hideTooltip}
          >
            <rect
              x={node.x0}
              y={node.y0}
              width="16"
              height={Math.max(1, (node.y1 ?? 0) - (node.y0 ?? 0))}
              fill={node.color}
            />
            <text
              x={node.side === "source"
                ? (node.x0 ?? 0) - 8
                : (node.x1 ?? 0) + 8}
              y={((node.y0 ?? 0) + (node.y1 ?? 0)) / 2}
              dy="0.35em"
              text-anchor={node.side === "source" ? "end" : "start"}
              >{node.name}</text
            >
          </g>
        {/each}
      </svg>
    </div>
  {:else}
    <p role="status">この月は線として表示できる正の報告値がありません。</p>
  {/if}
  <p>数値化できた報告値の卸売数量合計：{quantity(data.total)} t</p>
  <p>
    集計対象外のセル数：表示単位未満 {data.missing.below_display_unit}、事実なし {data
      .missing.no_fact}、不明・未収集 {data.missing
      .unknown_or_uncollected}。これらをゼロには置き換えていません。
  </p>
  <p>
    調査対象卸売市場の主要消費地域9区分の報告値を集計しています。下位地域は重複加算していません。農業生産量や全国の総流通量を示すものではありません。「産地」は委託者の都道府県という報告区分で、生産地と同義とは限りません。
  </p>
</div>

<style>
  .chart-container {
    min-width: 0;
  }
  .chart-scroll {
    overflow-x: auto;
    max-width: 100%;
  }
  .quantity-tooltip {
    position: fixed;
    z-index: 1000;
    width: max-content;
    max-width: min(320px, calc(100vw - 16px));
    box-sizing: border-box;
    padding: 0.75rem 1rem;
    border-radius: var(--radius-md);
    border: 1px solid #cbd5e1;
    background: #fff;
    color: #334155;
    font-size: 0.875rem;
    line-height: 1.6;
    box-shadow: 0 4px 16px rgb(51 65 85 / 14%);
    pointer-events: none;
  }
  .scroll-note {
    font-size: 0.875rem;
  }
  svg {
    display: block;
  }
  text {
    fill: var(--text-color, currentColor);
    font-size: 13px;
  }
  path {
    opacity: 0.35;
  }
  path:hover,
  path:focus {
    opacity: 0.8;
    outline: none;
  }
  g:focus {
    outline: none;
  }
  g:focus rect,
  g:hover rect {
    stroke: #94a3b8;
    stroke-width: 3;
  }
</style>
