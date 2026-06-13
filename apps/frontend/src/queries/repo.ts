import { queryOptions } from '@tanstack/react-query';
import { api } from '../utils/api';

export const repoQueryOptions = (team: string, repo: string) =>
	queryOptions({
		queryKey: ['repos', team, repo],
		queryFn: () =>
			api
				.GET('/api/v1/repos/{team}/{repo}', {
					params: {
						path: {
							repo,
							team,
						},
					},
				})
				.then((x) => x.data!),
	});
