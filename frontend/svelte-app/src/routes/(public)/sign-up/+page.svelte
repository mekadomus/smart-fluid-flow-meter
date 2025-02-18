<script lang="ts">
  import { goto } from '$app/navigation';
  import { zxcvbn } from '@zxcvbn-ts/core';

  import type { SignUpUserInput } from '@api/User';
  import { TurnstileSiteKey } from '@utils/Constants';
  import { signUpUser } from '@api/User';

  let captchaError = $state(false);

  async function signup(e: Event) {
    e.preventDefault();
    const formError = document.getElementById('error')!;
    formError.style.display = 'none';
    captchaError = false;

    const form = document.getElementById('signup-form') as HTMLFormElement;
    const email = document.getElementById('email') as HTMLFormElement;
    const name = document.getElementById('name') as HTMLFormElement;
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

    let token = '';
    if (!turnstile) {
      captchaError = true;
      return;
    } else {
      token = turnstile.getResponse();
      if (!token) {
        captchaError = true;
        return;
      }
    }

    const data: SignUpUserInput = {
      email: email.value,
      name: name.value,
      password: password.value,
      captcha: token
    };
    const res = await signUpUser(data);
    if ('email' in res) {
      goto('/sign-up-pending');
    } else {
      formError.style.display = 'block';
    }
  }
</script>

<svelte:head>
  <script src="https://challenges.cloudflare.com/turnstile/v0/api.js" defer></script>
</svelte:head>

<div class="signup">
  <h1>Sign-up</h1>
  <form action="#" method="POST" id="signup-form">
    <div class="form-group">
      <label for="email">Email</label>
      <input type="email" id="email" name="email" required />
    </div>
    <div class="form-group">
      <label for="name">Name</label>
      <input type="text" id="name" name="name" required />
    </div>
    <div class="form-group">
      <label for="password">Password</label>
      <input type="password" id="password" name="password" required />
    </div>
    <div class="form-group">
      <div
        class="cf-turnstile {captchaError ? 'captcha-error' : ''}"
        data-sitekey={TurnstileSiteKey}
        data-theme="light"
      ></div>
    </div>
    <div class="form-group">
      <div id="error" class="error-msg">
        Seems like we are experiencing some problems. We're working on fixing it.<br />
        We apologize for the inconvenience.
      </div>
      <button class="button" type="submit" onclick={(e: Event) => signup(e)}>Sign Up</button>
    </div>
    <div class="form-group">
      <a class="" href="/">I already have an account</a>
    </div>
  </form>
</div>

<style>
  .signup {
    border: 1px solid var(--primary-color);
    border-radius: 5px;
    margin: 0 auto;
    margin-top: 5rem;
    margin-bottom: 5rem;
    padding: 1rem;
    width: 24rem;
  }

  h1 {
    font-size: 1.3em;
    text-align: center;
  }

  a {
    margin-top: 3rem;
  }

  label,
  input {
    display: block;
  }

  input {
    border: 1px solid var(--primary-color);
    width: 100%;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .cf-turnstile {
    width: 300px;
    margin: 0 auto;
  }

  .captcha-error {
    border: 2px solid var(--secondary-color);
  }

  #error {
    margin-bottom: 15px;
    display: none;
  }
</style>
