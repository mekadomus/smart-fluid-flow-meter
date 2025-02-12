import { redirect } from '@sveltejs/kit';

import { AuthorizationCookie } from '@utils/Constants';
import { me } from '@api/User';

export async function load({ cookies }) {
  const token = cookies.get(AuthorizationCookie);
  let user;
  if (token) {
    user = await me(token);
    if (user && 'email' in user) {
      redirect(307, '/dashboard');
    }
  }

  return {};
}
