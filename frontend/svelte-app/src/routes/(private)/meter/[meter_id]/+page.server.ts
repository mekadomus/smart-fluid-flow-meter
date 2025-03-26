import type { PageServerLoad } from './$types';

import { AuthorizationCookie } from '@utils/Constants';
import { getMeasurements, getFluidMeterAlerts } from '@api/FluidMeter';

export const load: PageServerLoad = async ({ params, cookies }) => {
  const token = cookies.get(AuthorizationCookie);

  if (!token) {
    return {
      error: 'No authorization token'
    };
  }
  const [series, alerts] = await Promise.all([
    await getMeasurements(token, params.meter_id),
    await getFluidMeterAlerts(token, params.meter_id)
  ]);

  return {
    alerts,
    series
  };
};
