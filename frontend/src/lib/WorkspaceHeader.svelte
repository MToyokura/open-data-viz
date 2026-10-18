<script>
  import homeIcon from "#lib/assets/icons/home.svg";
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  let { title } = $props();
</script>

<header class="page-heading">
  <a href={`${VITE_ROOT_PATH}/`} aria-label="Home">
    <img src={homeIcon} alt="" />
  </a>
  <h1>{title}</h1>
</header>

<style>
  .page-heading {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: var(--space-4) var(--space-6);
    border-bottom: var(--border-default);
  }

  a {
    grid-column: 1;
    grid-row: 1;
    justify-self: start;
    color: inherit;
  }

  img {
    display: block;
    width: 24px;
    height: 24px;
  }

  h1 {
    grid-column: 2;
    grid-row: 1;
    justify-self: center;
    margin: 0;
    font-size: 1.25rem;
  }

  @media (max-width: 50rem) {
    .page-heading {
      padding: var(--space-3) var(--space-4);
    }

    h1 {
      font-size: 1.1rem;
    }
  }
</style>
