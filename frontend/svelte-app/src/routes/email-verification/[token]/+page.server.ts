import type { PageServerLoad } from './$types';
import { emailVerification } from '../../../lib/api/User';

export const load: PageServerLoad = async ({ params }) => {
  return {
    status: await emailVerification(params.token)
  };
};
