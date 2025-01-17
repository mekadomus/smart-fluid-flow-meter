<script lang="ts">
  type Props = {
    data: {
      status: number;
    };
  };
  let props: Props = $props();
  let status = $state(props.data.status);
</script>

{#if status >= 200 && status < 300}
  <div class="success-msg msg">
    Your e-mail has been verified. You can <a href="/">log in</a> now
  </div>
{:else}
  <div class="error-msg msg">The supplied token isn't valid</div>
{/if}

<style>
  .msg {
    margin: 0 auto;
    margin-top: 5rem;
    margin-bottom: 5rem;
    width: 34rem;
  }
</style>
