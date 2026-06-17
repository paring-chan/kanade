import { queryOptions } from '@tanstack/react-query';
import { api } from '../utils/api';

export const agentsQueryOptions = () =>
	queryOptions({
		queryKey: ['agents'],
		queryFn: () => api.GET('/api/v1/agents').then((x) => x.data!),
	});
