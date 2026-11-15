<script>
  /** @typedef {{ label: string, value?: string, id?: string, url?: string, linkLabel?: string, links?: { label: string, href: string }[] }} ReferenceRow */
  /** @type {{ rows: ReferenceRow[] }} */
  let { rows } = $props();
</script>

<table class="reference-table ui-table">
  <tbody>
    {#each rows.filter((row) => row.id || row.value || (row.url && row.linkLabel) || row.links?.length) as row, index (`${row.label}:${row.id ?? row.value ?? index}`)}
      <tr>
        <th scope="row">{row.label}</th>
        <td>
          {#if row.id}<div>{row.id}</div>{/if}
          {#if row.value}<div>{row.value}</div>{/if}
          {#if row.url && row.linkLabel}
            <div>
              <a href={row.url} target="_blank" rel="noopener noreferrer"
                >{row.linkLabel}</a
              >
            </div>
          {/if}
          {#each row.links ?? [] as link (link.href)}
            <div>
              <a href={link.href} target="_blank" rel="noopener noreferrer"
                >{link.label}</a
              >
            </div>
          {/each}
        </td>
      </tr>
    {/each}
  </tbody>
</table>
