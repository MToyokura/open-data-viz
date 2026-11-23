<script>
  import { page } from "$app/state";
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  let emojis = $state(["🚧🚧🚧🚧🚧"]);
  let containsKey = $state(false);
  const message = $derived(
    page.status === 404
      ? "お探しのページは見つかりませんでした。"
      : (page.error?.message ?? "ページの表示中にエラーが発生しました。"),
  );

  function addEmoji() {
    if (containsKey) return;
    const rand = Math.random();
    if (rand < 0.05) {
      emojis.push("🗝️");
      containsKey = true;
    } else if (rand < 0.2) {
      emojis.push("👷");
    } else {
      emojis.push("🚧");
    }
  }
</script>

<svelte:head>
  <title>{page.status} エラー | Open Data Viz</title>
</svelte:head>

<div class="page-shell">
  <main class="page-content">
    <section class="error-page" aria-labelledby="error-title">
      <h1 id="error-title">
        {page.status === 404
          ? "ページが見つかりません"
          : "ページを表示できません"}
      </h1>
      <p class="error-message">{message}</p>
      <p>URLを確認するか、ホームに戻ってください。</p>
      <p class="error-code">エラーコード: {page.status}</p>
      <a class="home-link" href={`${VITE_ROOT_PATH}/`}>ホームへ戻る</a>
    </section>

    <details class="easter-egg">
      <summary>おまけ</summary>
      <button
        type="button"
        onclick={addEmoji}
        aria-label="工事現場の絵文字を追加する"
      >
        {emojis.join("")}
      </button>
      {#if containsKey}
        <p role="status">
          鍵を発見しました！が何も起きません、ただのおまけです<span
            class="bikkuri">！</span
          >
        </p>
      {/if}
    </details>
  </main>
</div>

<style>
  .error-page {
    max-width: 42rem;
    margin: var(--space-8) auto;
    padding: var(--space-6);
    border-radius: var(--radius-hero);
    border: var(--border-default);
    color: var(--heading-color);
  }

  h1 {
    margin: 0 0 var(--space-4);
    font-size: 1.5rem;
  }

  .error-message {
    margin: 0 0 var(--space-3);
  }

  .error-code {
    color: var(--text-muted-color);
    font-size: 0.875rem;
  }

  .home-link {
    display: inline-block;
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-4);
    border: 1px solid var(--accent-color);
    border-radius: var(--radius-md);
    background: var(--surface-color);
    color: var(--text-color);
    text-decoration: none;
  }

  .home-link:hover {
    background: var(--surface-hover-color);
  }

  .easter-egg {
    max-width: 42rem;
    margin: 0 auto;
    color: var(--text-muted-color);
    font-size: 0.875rem;
  }

  .easter-egg summary {
    width: max-content;
    cursor: pointer;
  }

  .easter-egg button {
    margin-top: var(--space-2);
    padding: var(--space-1);
    border: 0;
    background: transparent;
    font-size: 1.5rem;
    cursor: pointer;
  }

  .easter-egg p {
    margin: var(--space-2) 0;
  }

  .bikkuri {
    display: inline-block;
    transition: transform 0.2s ease-in-out;
  }

  .bikkuri:hover {
    animation: rotate-bikkuri 0.8s linear infinite;
  }

  @keyframes rotate-bikkuri {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 700px) {
    .error-page {
      margin-block: var(--space-4);
      padding: var(--space-4);
    }

    .easter-egg {
      padding-inline: var(--space-2);
    }
  }
</style>
