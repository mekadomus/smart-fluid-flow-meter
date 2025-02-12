import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, url }) => {
  return {
    meter_id: params.meter_id,
    meter_name: url.searchParams.get('name')
  };
};
