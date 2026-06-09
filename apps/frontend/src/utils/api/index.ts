import type { paths } from './types';
import createClient from 'openapi-fetch';
import ky from 'ky';

export const api = createClient<paths>({ fetch: ky });
