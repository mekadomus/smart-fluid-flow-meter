<script lang="ts">
  import '../app.css';
  let { children } = $props();

  import MdHeader from '$lib/components/MdHeader.svelte';
  import MdNav from '$lib/components/MdNav.svelte';
</script>

<div>
  <MdHeader />
  <MdNav />
  <section>
    {@render children()}
  </section>
  <footer>All rights reserved</footer>
</div>
