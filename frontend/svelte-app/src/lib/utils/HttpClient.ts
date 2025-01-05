import { getCookie } from './Cookies';

import { AuthorizationCookie } from './Constants';
import { env } from '$env/dynamic/public';

function getToken(): string {
  return getCookie(AuthorizationCookie);
}

export async function httpGet(path: string) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  requestHeaders.set('Authorization', getToken());

  return fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'GET',
    headers: requestHeaders
  });
}

export async function httpPost(path: string, data: object) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  requestHeaders.set('Content-Type', 'application/json');
  requestHeaders.set('Authorization', getToken());

  return fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'POST',
    headers: requestHeaders,
    body: JSON.stringify(data)
  });
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
