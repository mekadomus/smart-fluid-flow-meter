<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext } from 'svelte';
  import { goto } from '$app/navigation';

  import { AuthorizationCookie } from '@utils/Constants';
  import { logOut } from '@api/User';
  import { deleteCookie } from '@utils/Cookies';

  const globalErrors: SvelteMap<string, string> = getContext('globalErrors');

  async function logout() {
    const status = await logOut();
    if (status == 200) {
      deleteCookie(AuthorizationCookie);
      goto('/');
    } else {
      globalErrors.set('sign-out-error', 'Sorry. There was an error.');
    }
  }
</script>

<header>
  <a href="/"><img alt="Mekadomus logo" src="/header-logo.png" width="150" height="85" /></a>
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
