import type { ErrorResponse } from './Error';
import { httpGet, httpPost } from '../utils/Http';
import { httpPostBrowser } from '../utils/HttpClient';

export type SignUpUserInput = {
  captcha: string;
  email: string;
  name: string;
  password: string;
};

export type LogInInput = {
  email: string;
  password: string;
};

enum UserAuthProvider {
  Password
}

export type User = {
  id: string;
  provider: UserAuthProvider;
  name: string;
  email: string;
  email_verified_at: Date;
  recorded_at: Date;
};

export async function signUpUser(input: SignUpUserInput): Promise<ErrorResponse | User> {
  const res = await httpPost('/v1/sign-up', input);
  return await res.json();
}

export async function logIn(input: LogInInput): Promise<ErrorResponse | User> {
  const res = await httpPost('/v1/log-in', input);
  return await res.json();
}

/**
 * Returns status code for the response
 */
export async function emailVerification(token: string): Promise<number> {
  const res = await httpGet(`/v1/email-verification?token=${token}`);
  return res.status;
}

/**
 * Returns information about the currently logged in user
 */
export async function me(auth_token: string): Promise<ErrorResponse | User> {
  const res = await httpGet(`/v1/me`, auth_token);
  return res.json();
}

/**
 * Logs the user out
 * Returns the status code of the response
 */
export async function logOut(): Promise<number> {
  const res = await httpPostBrowser(`/v1/log-out`, {});
  if ('code' in res) {
    return 500;
  }

  return 200;
}
