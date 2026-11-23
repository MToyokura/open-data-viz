<script>
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";

  let { children } = $props();
</script>

<div class="page-shell article-page">
  <WorkspaceHeader title="記事" asLabel />
  {@render children()}
</div>
