import { env } from '$env/dynamic/public';

export async function httpGet(path: string, token?: string) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  if (token) {
    requestHeaders.set('Authorization', token);
  }

  return fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'GET',
    headers: requestHeaders
  });
}

export async function httpPost(path: string, data: object) {
  const requestHeaders: HeadersInit = new Headers();
  requestHeaders.set('Accept', 'application/json');
  requestHeaders.set('Content-Type', 'application/json');

  return fetch(env.PUBLIC_BACKEND_URL + path, {
    method: 'POST',
    headers: requestHeaders,
    body: JSON.stringify(data)
  });
}
