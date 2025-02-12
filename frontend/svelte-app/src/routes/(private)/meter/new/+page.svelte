<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext } from 'svelte';
  import { goto } from '$app/navigation';

  import MdCenteredContainer from '@components/MdCenteredContainer.svelte';
  import type { CreateFluidMeterInput } from '@api/FluidMeter';
  import { createFluidMeter } from '@api/FluidMeter';
  import type { Message } from '@api/Message';
  import { MessageType } from '@api/Message';

  const globalMessages: SvelteMap<string, Message> = getContext('globalMessages');

  async function create() {
    const form = document.getElementById('new-meter-form') as HTMLFormElement;
    const name = document.getElementById('name') as HTMLFormElement;

    if (!form.checkValidity()) {
      form.reportValidity();
      return;
    }

    const data: CreateFluidMeterInput = {
      name: name.value
    };
    const res = await createFluidMeter(data);
    if ('owner_id' in res) {
      goto(`/meter/${res.id}/created?name=${encodeURI(res.name)}`);
    } else {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. There was an error adding the meter.'
      };
      globalMessages.set('new-meter-error', message);
    }
  }
</script>

<MdCenteredContainer>
  <h1>Create new meter</h1>
  <form action="#" method="POST" id="new-meter-form">
    <div class="form-group">
      <label for="name">Name</label>
      <input type="name" id="name" name="name" required />
    </div>

    <button class="button" type="button" onclick={() => create()}>Create</button>
  </form>
</MdCenteredContainer>

<style>
  h1 {
    margin: 0;
    padding-bottom: 1rem;
  }

  button {
    margin-top: 1rem;
  }
</style>
