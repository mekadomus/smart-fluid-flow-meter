<script lang="ts">
  import type { ErrorResponse } from '@api/Error';
  import type { FluidMeter } from '@api/FluidMeter';
  import type { PaginatedResponse } from '@api/Common';

  type Props = {
    data: {
      meters: PaginatedResponse<FluidMeter>;
      error: ErrorResponse;
    };
  };

  let props: Props = $props();
  let meters = $state(props.data.meters);
  let error = $state(props.data.error);
</script>

{#if error}
  <div class="error-msg msg">There was an error on our side. Sorry for the inconvenience.</div>
{/if}

{#if meters}
  {#if meters.items.length}
    Show items {meters.items.length}
  {:else}
    <div class="warning-msg msg">You currently don't own any meters</div>
    <a class="button" href="/meter/new">Create new meter</a>
  {/if}
{/if}

<style>
  .msg {
    margin: 1rem;
  }

  .button {
    display: block;
    width: 10rem;
    font-size: 0.9em;
  }
</style>
