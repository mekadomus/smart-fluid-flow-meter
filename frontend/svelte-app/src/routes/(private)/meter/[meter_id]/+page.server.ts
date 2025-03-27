import type { PageServerLoad } from './$types';

import { AuthorizationCookie } from '@utils/Constants';
import { getMeasurements, getFluidMeterAlerts } from '@api/FluidMeter';
import { SeriesGranularity } from '@api/Common';

export const load: PageServerLoad = async ({ params, cookies }) => {
  const token = cookies.get(AuthorizationCookie);

  if (!token) {
    return {
      error: 'No authorization token'
    };
  }
  const [series, alerts] = await Promise.all([
    await getMeasurements(token, params.meter_id, SeriesGranularity.Day, null),
    await getFluidMeterAlerts(token, params.meter_id)
  ]);

  return {
    alerts,
    series
  };
};
