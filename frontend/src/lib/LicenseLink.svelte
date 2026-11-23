<script>
  import CcIcon from "#lib/CcIcon.svelte";
  import CcZeroIcon from "#lib/CcZeroIcon.svelte";

  let { variant = "workspace" } = $props();
</script>

<a
  class="cc-icons cc-icons--{variant}"
  href="https://creativecommons.org/publicdomain/zero/1.0/deed.ja"
  target="_blank"
  rel="noreferrer"
  aria-label="CC0"
>
  <CcIcon size="2rem" />
  <CcZeroIcon size="2rem" />
</a>
