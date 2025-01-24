import type { ErrorResponse } from './Error';
import { httpGet } from '../utils/Http';
import { httpPostBrowser } from '../utils/HttpClient';

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
export async function getFluidMeters(token: string): Promise<FluidMeter[] | ErrorResponse> {
  const res = await httpGet(`/v1/fluid-meter`, token);
  return res.json();
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
