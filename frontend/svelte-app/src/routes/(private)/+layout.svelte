<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { setContext } from 'svelte';

  import '@src/app.css';

  let errors = $state(new SvelteMap());
  setContext('globalErrors', errors);

  let { children } = $props();

  import MdGlobalErrors from '@components/MdGlobalErrors.svelte';
  import MdPrivateHeader from '@components/MdPrivateHeader.svelte';
</script>

<div>
  <MdGlobalErrors {errors} />
  <MdPrivateHeader />
  <section>
    {@render children()}
  </section>
</div>
