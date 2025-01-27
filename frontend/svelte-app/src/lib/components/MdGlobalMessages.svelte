<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext } from 'svelte';

  import type { Message } from '@api/Message';

  const props = $props();
  const globalMessages: SvelteMap<string, Message> = getContext('globalMessages');

  function deleteMessage(id: string) {
    globalMessages.delete(id);
  }
</script>

<div class="container {props.messages.size ? '' : 'hidden'}">
  {#each [...props.messages] as [k, v]}
    <div class="box {v.type}-msg" id={k}>
      <p>{v.text}</p>
      <button onclick={() => deleteMessage(k)}>×</button>
    </div>
  {/each}
</div>

<style>
  .container {
    position: absolute;
    right: 1rem;
    top: 1rem;
    z-index: 100;
  }

  .box {
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
    font-weight: bold;
    cursor: pointer;
  }

  .error button {
    color: #cc0033;
  }

  .success button {
    color: #270;
  }

  .warning button {
    color: #9f6000;
  }
</style>
