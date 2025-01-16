<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext } from 'svelte';

  const props = $props();
  const globalErrors: SvelteMap<string, string> = getContext('globalErrors');

  function deleteError(id: string) {
    globalErrors.delete(id);
  }
</script>

<div class="container {props.errors.size ? '' : 'hidden'}">
  {#each [...props.errors] as [k, v]}
    <div class="error" id={k}>
      <p>{v}</p>
      <button onclick={() => deleteError(k)}>×</button>
    </div>
  {/each}
</div>

<style>
  .container {
    position: absolute;
    right: 1rem;
    top: 1rem;
    border-radius: 5px;
    border: 1px solid #fcc2c3;
    padding: 10px 15px;
    font-size: 0.9em;
    background-color: rgba(252, 228, 228, 0.9);
    color: #cc0033;
    z-index: 100;
  }

  .error {
    display: flex;
  }

  p {
    vertical-align: center;
  }

  button {
    margin-left: 1rem;
    font-size: 2em;
    border: none;
    background: none;
    color: #cc0033;
    font-weight: bold;
    cursor: pointer;
  }
</style>
