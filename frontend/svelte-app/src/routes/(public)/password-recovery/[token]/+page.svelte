<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext } from 'svelte';
  import { goto } from '$app/navigation';
  import { zxcvbn } from '@zxcvbn-ts/core';

  import type { Message } from '@api/Message';
  import type { NewPasswordInput } from '@api/User';

  import MdCenteredContainer from '@components/MdCenteredContainer.svelte';
  import { MessageType } from '@api/Message';
  import { setNewPassword } from '@api/User';

  const globalMessages: SvelteMap<string, Message> = getContext('globalMessages');

  type Props = {
    data: {
      token: string;
    };
  };
  let props: Props = $props();

  async function newPassword(e: Event) {
    e.preventDefault();

    const form = document.getElementById('np-form') as HTMLFormElement;
    const password = document.getElementById('password') as HTMLFormElement;

    password.setCustomValidity('');
    form.reportValidity();

    if (!form.checkValidity()) {
      form.reportValidity();
      return;
    }

    if (zxcvbn(password.value).score < 3) {
      password.setCustomValidity('Password is too weak');
      form.reportValidity();
      return;
    }

    const data: NewPasswordInput = {
      token: props.data.token,
      password: password.value
    };
    const status = await setNewPassword(data);
    if (status == 200) {
      goto('/');
    } else if (status == 400) {
      let message: Message = {
        type: MessageType.Error,
        text: 'The reset token is not valid'
      };
      globalMessages.set('new-password-error', message);
    } else {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. There was an error resetting your password.'
      };
      globalMessages.set('new-password-error', message);
    }
  }
</script>

<MdCenteredContainer>
  <h1>Choose your new password</h1>
  <form action="#" method="POST" id="np-form">
    <div class="form-group">
      <label for="password">Password</label>
      <input type="password" id="password" name="password" required />
    </div>
    <div class="form-group">
      <button class="button" type="submit" onclick={(e: Event) => newPassword(e)}
        >Update password</button
      >
    </div>
  </form>
</MdCenteredContainer>

<style>
  button {
    margin-top: 1rem;
  }
</style>
