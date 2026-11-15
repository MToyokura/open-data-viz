<script>
  import Spinner from "#lib/Spinner.svelte";

  /** @type {{ state: "loading" | "error", message: string, onretry?: () => void, retryLabel?: string }} */
  let { state, message, onretry, retryLabel = "再試行" } = $props();
</script>

{#if state === "loading"}
  <div class="load-state" role="status">
    <Spinner />
    <p>{message}</p>
  </div>
{:else}
  <div class="load-state error" role="alert">
    <p>{message}</p>
    {#if onretry}
      <button type="button" onclick={onretry}>{retryLabel}</button>
    {/if}
  </div>
{/if}

<style>
  .load-state {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin: var(--space-3) 0;
  }

  .load-state p {
    margin: 0;
  }
</style>
