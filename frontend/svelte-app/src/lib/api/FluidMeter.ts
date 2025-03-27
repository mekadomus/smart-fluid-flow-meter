import type { Alert } from '@api/Alert';
import type { ErrorResponse } from './Error';
import type { PaginatedResponse, Series, SeriesGranularity } from './Common';

import { PAGE_SIZE } from './Common';
import { httpGet } from '../utils/Http';
import { httpPostBrowser, httpGetBrowser, httpDeleteBrowser } from '../utils/HttpClient';

export enum FluidMeterStatus {
  Active = 'Active',
  // Still shown to the user, but not triggering alarms
  Inactive = 'Inactive',
  // Not shown to the user
  Deleted = 'Deleted'
}

export type FluidMeter = {
  id: string;
  name: string;
  owner_id: string;
  status: FluidMeterStatus;
  recorded_at: Date;
};

export type FluidMeterAlerts = {
  meter: FluidMeter;
  alerts: Alert[];
};

export type CreateFluidMeterInput = {
  name: string;
};

/**
 * Get information about a fluid meter
 */
export async function getFluidMeter(
  token: string,
  meter_id: string
): Promise<FluidMeter | ErrorResponse> {
  const res = await httpGet(`/v1/fluid-meter/${meter_id}`, token);
  return res.json();
}

/**
 * Get all alerts for a fluid meter
 */
export async function getFluidMeterAlerts(
  token: string,
  meter_id: string
): Promise<FluidMeterAlerts | ErrorResponse> {
  const res = await httpGet(`/v1/fluid-meter/${meter_id}/alert`, token);
  return res.json();
}

/**
 * Deactivate a fluid meter
 */
export async function deactivateFluidMeter(meter_id: string): Promise<number> {
  const res = await httpPostBrowser(`/v1/fluid-meter/${meter_id}/deactivate`, {});

  if (res && 'code' in res) {
    if (res.code == 'InternalError') {
      return 500;
    } else {
      return 400;
    }
  }

  return 200;
}

/**
 * Deletes a fluid meter
 */
export async function deleteFluidMeter(meter_id: string): Promise<number> {
  const res = await httpDeleteBrowser(`/v1/fluid-meter/${meter_id}`);

  if (res && 'code' in res) {
    if (res.code == 'InternalError') {
      return 500;
    } else {
      return 400;
    }
  }

  return 200;
}

/**
 * Activate a fluid meter
 */
export async function activateFluidMeter(meter_id: string): Promise<number> {
  const res = await httpPostBrowser(`/v1/fluid-meter/${meter_id}/activate`, {});

  if (res && 'code' in res) {
    if (res.code == 'InternalError') {
      return 500;
    } else {
      return 400;
    }
  }

  return 200;
}

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
  return await httpPostBrowser(`/v1/fluid-meter`, input);
}

/**
 * Get the measurements for the given meter
 */
export async function getMeasurements(
  token: string,
  meter: string,
  granularity: SeriesGranularity,
  day: Date | null
): Promise<Series | ErrorResponse> {
  let url = `/v1/fluid-meter/${meter}/measurement?granularity=${granularity}`;
  if (day) {
    url += `&day=${day}`;
  }
  const res = await httpGet(url, token);
  return res.json().catch(() => {
    return {
      code: 'InternalError',
      message: 'We encountered an error'
    };
  });
}

/**
 * Get the measurements for the given meter
 */
export async function getMeasurementsBrowser(
  meter: string,
  granularity: SeriesGranularity,
  day: Date | null
): Promise<Series | ErrorResponse> {
  let url = `/v1/fluid-meter/${meter}/measurement?granularity=${granularity}`;
  if (day) {
    url += `&day=${day.toISOString().split('T')[0]}`;
  }
  return await httpGetBrowser(url);
}
