<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { setContext } from 'svelte';

  import '@src/app.css';
  import MdGlobalMessages from '@components/MdGlobalMessages.svelte';
  import MdHeader from '@components/MdHeader.svelte';

  let { children } = $props();

  let messages = $state(new SvelteMap());
  setContext('globalMessages', messages);
</script>

<div>
  <MdGlobalMessages {messages} />
  <MdHeader />
  <section>
    {@render children()}
  </section>
</div>
