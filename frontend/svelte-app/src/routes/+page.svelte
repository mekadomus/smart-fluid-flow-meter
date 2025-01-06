<script lang="ts">
  import { TurnstileSiteKey } from '../lib/utils/Constants';
  import { httpPost } from '../lib/utils/HttpClient';
  import { zxcvbn } from '@zxcvbn-ts/core';

  let captchaError = $state(false);

  async function login(e: Event) {
    e.preventDefault();
    captchaError = false;

    const form = document.getElementById('login-form') as HTMLFormElement;
    const email = document.getElementById('email') as HTMLFormElement;
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

    const data = {
      email: email.value,
      password: password.value,
      captcha: token
    };
    const res = await httpPost('/v1/sign-up', data);
    if (res.status === 201 || res.status == 200) {
      alert('good');
    } else {
      alert('bad');
    }
  }
</script>

<svelte:head>
  <script src="https://challenges.cloudflare.com/turnstile/v0/api.js" defer></script>
</svelte:head>

<div class="login">
  <h1>Login</h1>
  <form action="#" method="POST" id="login-form">
    <div class="form-group">
      <label for="email">Email</label>
      <input type="email" id="email" name="email" required />
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
      <button type="submit" onclick={(e: Event) => login(e)}>Log In</button>
    </div>
  </form>
</div>

<style>
  .login {
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
    padding-bottom: 1.5rem;
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

  button {
    background: var(--primary-color);
    color: var(--secondary-color-weak);
    border-radius: 5px;
    display: block;
    margin: 0 auto;
    padding: 0.5rem;
  }
</style>
