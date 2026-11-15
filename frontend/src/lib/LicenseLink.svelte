<script>
  import CcIcon from "#lib/CcIcon.svelte";
  import CcZeroIcon from "#lib/CcZeroIcon.svelte";

  let { variant = "workspace" } = $props();
</script>

<a
  class={[
    "cc-icons",
    { workspace: variant === "workspace", footer: variant === "footer" },
  ]}
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
    flex-flow: row nowrap;
    justify-content: end;
    align-items: center;
    gap: var(--space-1);
    width: max-content;
    color: var(--text-color);
    text-decoration: none;
  }

  .cc-icons :global(svg) {
    display: block;
    flex: 0 0 2rem;
  }

  .workspace {
    margin: var(--space-4) 0 var(--space-4) auto;
    padding: var(--space-2);
    border-radius: var(--radius-md);
    background: var(--surface-color);
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
