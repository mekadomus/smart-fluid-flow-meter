<script lang="ts">
  import '@src/app.css';
  let { children } = $props();

  import MdHeader from '@components/MdHeader.svelte';
</script>

<div>
  <MdHeader />
  <section>
    {@render children()}
  </section>
</div>
