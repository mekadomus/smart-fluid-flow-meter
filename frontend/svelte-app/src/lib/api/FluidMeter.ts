import type { ErrorResponse } from './Error';
import type { PaginatedResponse } from './Common';

import { PAGE_SIZE } from './Common';
import { httpGet } from '../utils/Http';
import { httpPostBrowser, httpGetBrowser } from '../utils/HttpClient';

enum FluidMeterStatus {
  Active,
  // Still shown to the user, but not triggering alarms
  Inactive,
  // Not shown to the user
  Deleted
}

export type FluidMeter = {
  id: string;
  name: string;
  owner_id: string;
  status: FluidMeterStatus;
  recorded_at: Date;
};

export type CreateFluidMeterInput = {
  name: string;
};

/**
 * Get a page of fluid meters
 */
export async function getFluidMeters(
  token: string
): Promise<PaginatedResponse<FluidMeter> | ErrorResponse> {
  const res = await httpGet(`/v1/fluid-meter?page_size=${PAGE_SIZE}`, token);
  return res.json();
}

/**
 * Get a page of fluid meters
 */
export async function getFluidMetersBrowser(
  after: string
): Promise<PaginatedResponse<FluidMeter> | ErrorResponse> {
  return httpGetBrowser(`/v1/fluid-meter?page_size=${PAGE_SIZE}&page_cursor=${after}`);
}

/**
 * Create a new fluid meter for current user
 */
export async function createFluidMeter(
  input: CreateFluidMeterInput
): Promise<FluidMeter | ErrorResponse> {
  const res = await httpPostBrowser(`/v1/fluid-meter`, input);
  return res.json();
}
