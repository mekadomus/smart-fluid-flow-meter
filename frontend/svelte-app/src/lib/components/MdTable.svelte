<script>
  let props = $props();
</script>

<table>
  {#if props.headers}
    <thead>
      <tr>
        {#each props.headers as h}
          <th>{h}</th>
        {/each}
      </tr>
    </thead>
  {/if}
  <tbody>
    {#each props.items as item}
      <tr>
        {#each Object.values(item) as v}
          <td>{@html v}</td> <!-- eslint-disable-line svelte/no-at-html-tags -->
        {/each}
      </tr>
    {/each}
  </tbody>
</table>
<nav>
  {#if props.moreCallback}
    <button type="button" onclick={props.moreCallback}>More</button>
  {/if}
  {#if props.lessCallback}
    <button type="button" onclick={props.lessCallback}>More</button>
  {/if}
</nav>

<style>
  table {
    border-collapse: collapse;
  }

  th {
    background: var(--primary-color);
    color: var(--secondary-color-weak);
  }

  th,
  td {
    padding: 0.1rem 1rem;
    border: none;
  }
</style>
