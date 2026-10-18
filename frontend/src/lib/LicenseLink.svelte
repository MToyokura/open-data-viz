<script>
  import CcIcon from "#lib/CcIcon.svelte";
  import CcZeroIcon from "#lib/CcZeroIcon.svelte";

  let { variant = "workspace" } = $props();
</script>

<a
  class:workspace={variant === "workspace"}
  class:footer={variant === "footer"}
  class="cc-icons"
  href="https://creativecommons.org/publicdomain/zero/1.0/deed.ja"
  target="_blank"
  rel="noreferrer"
  aria-label="CC0"
>
  <CcIcon size="2rem" />
  <CcZeroIcon size="2rem" />
</a>

<style>
  .cc-icons {
    display: flex;
    justify-content: end;
    align-items: center;
    gap: var(--space-1);
    width: max-content;
    color: #111;
    text-decoration: none;
  }

  .workspace {
    margin: var(--space-4) 0 var(--space-4) auto;
    padding: var(--space-2);
    border-radius: var(--radius-md);
    background: #fff;
  }

  .workspace :global(svg) {
    opacity: 0.5;
    transition: opacity 0.2s ease-in-out;
  }

  .workspace:hover :global(svg) {
    opacity: 1;
  }

  .footer {
    opacity: 0.5;
    transition: opacity 0.2s ease-in-out;
  }

  .footer:hover {
    opacity: 1;
  }
</style>
