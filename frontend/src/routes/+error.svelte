<script lang="ts">
  import { page } from "$app/state";
  let emojis = $state<string[]>(["🚧🚧🚧🚧🚧"]);
  let containsKey = $state<boolean>(false);

  function addEmoji() {
    if (containsKey) {
      return;
    }
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

<h1>{`${page.status} ${page.error?.message ?? ""}`}</h1>
<button type="button" onclick={addEmoji} class="construction">
  {emojis.join("")}
</button>
{#if containsKey}
  <p>
    鍵を発見しました！が何も起きません、ただのおまけです<button
      type="button"
      class="bikkuri">！</button
    >
  </p>
{/if}

<style>
  .construction {
    all: unset;
    cursor: pointer;
    user-select: none;
    align-self: flex-start;
  }

  .bikkuri {
    all: unset;
    cursor: default;
    transition: font-size 0.2s;
    align-self: flex-start;
  }

  @keyframes rotate-bikkuri {
    100% {
      transform: rotate(360deg);
    }
  }

  .bikkuri:hover {
    animation: rotate-bikkuri 0.8s linear infinite;
  }
</style>
