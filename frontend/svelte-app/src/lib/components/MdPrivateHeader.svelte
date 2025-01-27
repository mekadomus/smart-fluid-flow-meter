<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext } from 'svelte';
  import { goto } from '$app/navigation';

  import { AuthorizationCookie } from '@utils/Constants';
  import type { Message } from '@api/Message';
  import { MessageType } from '@api/Message';
  import { deleteCookie } from '@utils/Cookies';
  import { logOut } from '@api/User';

  const globalMessages: SvelteMap<string, Message> = getContext('globalMessages');

  async function logout() {
    const status = await logOut();
    if (status == 200) {
      deleteCookie(AuthorizationCookie);
      goto('/');
    } else {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. There was an error.'
      };
      globalMessages.set('sign-out-error', message);
    }
  }
</script>

<header>
  <a href="/dashboard"
    ><img alt="Mekadomus logo" src="/header-logo.png" width="150" height="85" /></a
  >
  <button id="log-out" onclick={() => logout()}>Log out</button>
</header>

<style>
  header {
    border-bottom: 2px solid var(--primary-color);
    padding: 0.5rem 2rem;
    color: var(--secondary-color-weak);
    display: flex;
    justify-content: space-between;
  }

  button {
    border: none;
    background: none;
    font-size: 1.2em;
    color: var(--primary-color);
    font-weight: bold;
    cursor: pointer;
  }

  button:hover {
    filter: saturate(0.5);
  }
</style>
