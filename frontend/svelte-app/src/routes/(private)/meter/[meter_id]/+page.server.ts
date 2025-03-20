import type { PageServerLoad } from './$types';

import { AuthorizationCookie } from '@utils/Constants';
import { getMeasurements, getFluidMeter } from '@api/FluidMeter';

export const load: PageServerLoad = async ({ params, cookies }) => {
  const token = cookies.get(AuthorizationCookie);

  if (!token) {
    return {
      error: 'No authorization token'
    };
  }
  const [series, meter_data] = await Promise.all([
    await getMeasurements(token, params.meter_id),
    await getFluidMeter(token, params.meter_id)
  ]);

  return {
    meter_data,
    series
  };
};
