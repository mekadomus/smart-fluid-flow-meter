import { getCookie } from './Cookies';

import { AuthorizationCookie } from './Constants';
import { env } from '$env/dynamic/public';

function getToken(): string {
  return getCookie(AuthorizationCookie);
}

export async function httpGetBrowser(path: string) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  requestHeaders.set('Authorization', getToken());

  const r = await fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'GET',
    headers: requestHeaders
  });

  if (r.ok) {
    return r.json();
  } else {
    return {
      code: 'InternalError',
      message: 'We encountered an error'
    };
  }
}

export async function httpDeleteBrowser(path: string) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  requestHeaders.set('Authorization', getToken());

  const r = await fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'DELETE',
    headers: requestHeaders
  });

  if (r.ok) {
    return r.json();
  } else {
    try {
      return r.json();
    } catch {
      return {
        code: 'InternalError',
        message: 'We encountered an error'
      };
    }
  }
}

export async function httpPostBrowser(path: string, data: object) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  requestHeaders.set('Content-Type', 'application/json');
  requestHeaders.set('Authorization', getToken());

  const r = await fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'POST',
    headers: requestHeaders,
    body: JSON.stringify(data)
  });

  if (r.ok) {
    return r.json();
  } else {
    try {
      return r.json();
    } catch {
      return {
        code: 'InternalError',
        message: 'We encountered an error'
      };
    }
  }
}

export async function httpPut(path: string, data: object) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  requestHeaders.set('Content-Type', 'application/json');
  requestHeaders.set('Authorization', getToken());

  return fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'PUT',
    headers: requestHeaders,
    body: JSON.stringify(data)
  });
}
