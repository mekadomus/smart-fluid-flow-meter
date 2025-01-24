<script>
  let { children } = $props();
</script>

<section>
  {@render children()}
</section>

<style>
  section {
    border: 1px solid var(--primary-color);
    border-radius: 5px;
    margin: 0 auto;
    margin-top: 5rem;
    margin-bottom: 5rem;
    padding: 1rem;
    width: 24rem;
  }
</style>
