<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { setContext } from 'svelte';

  import '@src/app.css';

  let messages = $state(new SvelteMap());
  setContext('globalMessages', messages);

  let { children } = $props();

  import MdGlobalMessages from '@components/MdGlobalMessages.svelte';
  import MdPrivateHeader from '@components/MdPrivateHeader.svelte';
  import MdPrivateNavigation from '@components/MdPrivateNavigation.svelte';
</script>

<div>
  <MdGlobalMessages {messages} />
  <MdPrivateHeader />
  <MdPrivateNavigation />
  <section>
    {@render children()}
  </section>
</div>
