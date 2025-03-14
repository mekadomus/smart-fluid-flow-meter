import type { PageServerLoad } from './$types';

import { AuthorizationCookie } from '@utils/Constants';
import { getMeasurements } from '@api/FluidMeter';

export const load: PageServerLoad = async ({ params, cookies }) => {
  const token = cookies.get(AuthorizationCookie);

  if (!token) {
    return {
      error: 'No authorization token'
    };
  }

  return {
    meter_id: params.meter_id,
    series: await getMeasurements(token, params.meter_id)
  };
};
