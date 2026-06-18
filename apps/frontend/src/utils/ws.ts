import { pipelineJobsQueryOptions, pipelineQueryOptions } from '../queries/pipeline';
import type { EventMessage } from '../ws-types';
import { queryClient } from './api';

export class EventSocket {
	private ws?: WebSocket;

	constructor() {
		this.connect();
	}

	connect() {
		if (this.ws) {
			this.ws.close();
		}

		const loc = window.location;
		const ws = new WebSocket(`${loc.protocol.replace('http', 'ws')}//${loc.host}/_/ws/events`);
		this.ws = ws;

		ws.onopen = () => {
			console.log('ws connected');
			queryClient.invalidateQueries({ refetchType: 'active' });
		};

		ws.onmessage = (ev) => {
			try {
				const data = JSON.parse(ev.data) as EventMessage;
				console.log(data);

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
						console.warn('unknown data type:', data.t);
						return;
				}
			} catch (e) {
				console.error('failed to parse message:', ev.data, e);
			}
		};

		ws.onclose = () => {
			console.log('connection closed');

			setTimeout(() => {
				console.log('reconnecting...');
				this.connect();
			}, 1000);
		};
	}
}

export const eventSocket = new EventSocket();
