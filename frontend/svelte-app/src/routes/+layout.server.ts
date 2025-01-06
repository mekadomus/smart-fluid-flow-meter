import { redirect } from '@sveltejs/kit';

import { AuthorizationCookie, PublicPaths } from '../lib/utils/Constants';
import { me } from '../lib/api/User';

export async function load({ url, cookies }) {
  const token = cookies.get(AuthorizationCookie);
  let user;
  if (token) {
    user = await me(token);
  }

  if (!user || !('email' in user)) {
    cookies.delete(AuthorizationCookie, { path: '/' });
    if (!PublicPaths.has(url.pathname)) {
      redirect(307, '/');
    }
  }

  return {
    session: {
      token,
      user
    }
  };
}
