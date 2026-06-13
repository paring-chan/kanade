import { queryOptions } from '@tanstack/react-query';
import { api } from '../utils/api';

export const teamListQueryOptions = () =>
	queryOptions({
		queryKey: ['teams'],
		queryFn: () => api.GET('/api/v1/teams').then((x) => x.data!),
	});

export const teamBySlugQueryOptions = (slug: string) =>
	queryOptions({
		queryKey: ['teams', 'by-slug', slug],
		queryFn: () =>
			api
				.GET('/api/v1/teams/{team_slug}', {
					params: { path: { team_slug: slug } },
				})
				.then((x) => x.data!),
	});

export const teamReposQueryOptions = (teamSlug: string) =>
	queryOptions({
		queryKey: ['teams', 'by-slug', teamSlug, 'repos'],
		queryFn: () =>
			api
				.GET('/api/v1/teams/{team_slug}/repos', {
					params: { path: { team_slug: teamSlug } },
				})
				.then((x) => x.data!),
	});
