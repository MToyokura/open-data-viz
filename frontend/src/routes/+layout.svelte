<script lang="ts">
  import CcIcon from "#lib/CcIcon.svelte";
  import CcZeroIcon from "#lib/CcZeroIcon.svelte";
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  let { children } = $props();
</script>

<svelte:head>
  <title>Open Data Viz</title>
  <meta
    name="description"
    content="データの加工から可視化までのすべての工程を公開して、自由に利用できるようにすることをコンセプトに趣味で始めたプロジェクトです"
  />
</svelte:head>

<nav class="site-nav">
  <a href={`${VITE_ROOT_PATH}/`}>Home</a>
</nav>

<div id="svelte-main">
  {@render children()}
</div>

<footer class="site-footer">
  <a
    class="cc-icons"
    href="https://creativecommons.org/publicdomain/zero/1.0/deed.ja"
    target="_blank"
    rel="noreferrer"
  >
    <CcIcon size="2rem" />
    <CcZeroIcon size="2rem" />
  </a>
</footer>

<style>
  .site-nav {
    padding: var(--space-3) var(--page-content-padding);
  }

  .site-footer {
    display: flex;
    justify-content: end;
    margin: var(--space-8) var(--space-4) var(--space-4);
  }

  .cc-icons {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: var(--space-1);
    text-decoration: none;
    color: inherit;
    opacity: 0.5;
    transition: opacity 0.2s ease-in-out;
  }

  .cc-icons:hover {
    opacity: 1;
  }
</style>
