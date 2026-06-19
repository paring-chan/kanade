import { pipelineJobsQueryOptions, pipelineQueryOptions } from '../queries/pipeline';
import type { EventMessage } from '../ws-types';
import { queryClient } from './api';

export class EventSocket {
	// private sse?: EventSource;

	constructor() {
		this.connect();
	}

	connect() {
		const sse = new EventSource(`/_/sse/events`);
		// this.sse = sse;

		sse.addEventListener('open', () => {
			console.log('ws connected');
			queryClient.invalidateQueries({ refetchType: 'active' });
		});

		sse.addEventListener('message', (ev) => {
			try {
				const data = JSON.parse(ev.data) as EventMessage;

				switch (data.t) {
					case 'updatePipelineStatus': {
						const key = pipelineQueryOptions(data.p.pipeline).queryKey;
						const prevData = queryClient.getQueryData(key);
						if (prevData) {
							queryClient.setQueryData(key, {
								...prevData,
								status: data.p.status,
							});
						}
						break;
					}
					case 'updateJobStatus': {
						const key = pipelineJobsQueryOptions(data.p.pipeline).queryKey;
						const prevData = queryClient.getQueryData(key);
						if (prevData) {
							const target = [...prevData];

							for (let i = 0; i < target.length; i++) {
								const job = target[i]!;
								if (job.id === data.p.job) {
									target[i] = { ...job, status: data.p.status };
									break;
								}
							}

							queryClient.setQueryData(key, target);
						}

						break;
					}

					default:
						console.warn('unknown data type:', data);
						return;
				}
			} catch (e) {
				console.error('failed to parse message:', ev.data, e);
			}
		});
	}
}

export const eventSocket = new EventSocket();
