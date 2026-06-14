import { queryOptions } from '@tanstack/react-query';
import { api } from '../utils/api';

export const pipelineQueryOptions = (pipeline: string) =>
	queryOptions({
		queryKey: ['pipelines', pipeline],
		queryFn: () =>
			api
				.GET('/api/v1/pipelines/{pipeline_id}', {
					params: {
						path: { pipeline_id: pipeline },
					},
				})
				.then((x) => x.data!),
	});

export const pipelineJobsQueryOptions = (pipeline: string) =>
	queryOptions({
		queryKey: ['pipelines', pipeline, 'jobs'],
		queryFn: () =>
			api
				.GET('/api/v1/pipelines/{pipeline_id}/jobs', {
					params: {
						path: { pipeline_id: pipeline },
					},
				})
				.then((x) => x.data!),
	});
