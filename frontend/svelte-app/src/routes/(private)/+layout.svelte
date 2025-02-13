<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { setContext } from 'svelte';

  import '@src/app.css';
  import MdGlobalMessages from '@components/MdGlobalMessages.svelte';
  import MdPrivateHeader from '@components/MdPrivateHeader.svelte';
  import MdPrivateNavigation from '@components/MdPrivateNavigation.svelte';

  let messages = $state(new SvelteMap());
  setContext('globalMessages', messages);

  let { children } = $props();
</script>

<div>
  <MdGlobalMessages {messages} />
  <MdPrivateHeader />
  <MdPrivateNavigation />
  <section>
    {@render children()}
  </section>
</div>
