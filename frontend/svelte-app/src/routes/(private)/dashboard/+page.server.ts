import type { PageServerLoad } from './$types';

import { AuthorizationCookie } from '@utils/Constants';
import { getFluidMeters } from '@api/FluidMeter';

export const load: PageServerLoad = async ({ cookies }) => {
  const token = cookies.get(AuthorizationCookie);

  if (!token) {
    return {
      error: 'No authorization token'
    };
  }

  const meters = await getFluidMeters(token);

  if ('items' in meters) {
    return {
      meters
    };
  } else {
    return {
      error: meters
    };
  }
};
