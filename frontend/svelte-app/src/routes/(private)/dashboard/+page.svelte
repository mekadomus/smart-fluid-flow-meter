<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext } from 'svelte';

  import type { ErrorResponse } from '@api/Error';
  import type { FluidMeter } from '@api/FluidMeter';
  import type { Message } from '@api/Message';
  import type { PaginatedResponse } from '@api/Common';

  import MdTable from '@components/MdTable.svelte';
  import { MessageType } from '@api/Message';
  import { getFluidMetersBrowser } from '@api/FluidMeter';

  const globalMessages: SvelteMap<string, Message> = getContext('globalMessages');

  type Props = {
    data: {
      meters: PaginatedResponse<FluidMeter>;
      error: ErrorResponse;
    };
  };

  let props: Props = $props();

  const meters = props.data.meters;
  let i = meters.items.map((v) => {
    return [v.name, v.status, new Date(v.recorded_at).toLocaleString()];
  });
  let items = $state(i);
  let error = $state(props.data.error);
  let hasMore: (() => void) | null = $state(null);

  async function loadMore(after: string) {
    let r = await getFluidMetersBrowser(after);
    if ('items' in r) {
      let i = r.items.map((v) => {
        return [v.name, v.status, new Date(v.recorded_at).toLocaleString()];
      });
      items = i;

      console.log(r.pagination);
      if (r.pagination.has_more) {
        hasMore = () => {
          loadMore(r.items[r.items.length - 1].id);
        };
      } else {
        hasMore = null;
      }
    } else {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. There was an error getting your meters.'
      };
      globalMessages.set('new-meter-error', message);
    }
  }

  let pagination = meters.pagination;
  if (pagination) {
    if (pagination.has_more) {
      hasMore = () => {
        loadMore(meters.items[meters.items.length - 1].id);
      };
    }
  }
</script>

{#if error}
  <div class="error-msg msg">There was an error on our side. Sorry for the inconvenience.</div>
{/if}

{#if items}
  {#if items.length}
    <div class="container">
      <MdTable {items} headers={['Name', 'Status', 'Creation date']} moreCallback={hasMore} />
    </div>
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

  .container {
    margin: 0 auto;
    margin-top: 1rem;
    width: 80%;
  }
</style>
