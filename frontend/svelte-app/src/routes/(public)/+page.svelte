<script lang="ts">
  import { goto } from '$app/navigation';
  import { zxcvbn } from '@zxcvbn-ts/core';

  import type { LogInInput } from '@api/User';
  import { AuthorizationCookie } from '@utils/Constants';
  import { ErrorCode } from '@api/Error';
  import { logIn } from '@api/User';
  import { setCookie } from '@utils/Cookies';

  async function login(e: Event) {
    e.preventDefault();
    const internalError = document.getElementById('internal-error')!;
    const clientError = document.getElementById('client-error')!;
    internalError.style.display = 'none';
    clientError.style.display = 'none';

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

    const data: LogInInput = {
      email: email.value,
      password: password.value
    };
    const res = await logIn(data);
    if ('token' in res) {
      setCookie(AuthorizationCookie, 'Bearer ' + res.token);
      goto('/dashboard');
    } else {
      if ('code' in res && res.code == ErrorCode[ErrorCode.ValidationError]) {
        clientError.style.display = 'block';
      } else {
        internalError.style.display = 'block';
      }
    }
  }
</script>

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
      <div id="internal-error" class="error-msg">
        Seems like we are experiencing some problems. We're working on fixing it.<br />
        We apologize for the inconvenience.
      </div>
      <div id="client-error" class="error-msg">Credentials are not valid. Try again.</div>
      <button class="button" type="submit" onclick={(e: Event) => login(e)}>Log In</button>
    </div>
    <div class="form-group links">
      <a class="" href="/password-recovery">Forgot my password</a><br />
      <a class="" href="/sign-up">I don't have an account</a>
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

  .error-msg {
    margin-bottom: 15px;
    display: none;
  }

  .links {
    margin-top: 3rem;
    font-size: 0.9em;
  }
</style>
