import { createBrowserRouter, type RouteObject } from 'react-router';

const routes = [
	{
		lazy: () => import('./layouts/root'),
		children: [
			{
				lazy: () => import('./layouts/normal'),
				children: [
					{ path: 'login', lazy: () => import('./routes/login') },
					{ index: true, lazy: () => import('./routes/project-list') },
					{ path: 'login/success', lazy: () => import('./routes/login-success') },
					{ path: 'teams', lazy: () => import('./routes/team-list') },
					{ path: 't/:team', lazy: () => import('./routes/team-info') },
					{ path: 'r/:team/:repo', lazy: () => import('./routes/repo-info') },
					{ path: 't/:team/secrets', lazy: () => import('./routes/team-secret-list') },
					{ path: 'r/:team/:repo/secrets', lazy: () => import('./routes/repo-secret-list') },

					{ path: 'p/:pipeline', lazy: () => import('./routes/pipeline-view') },

					{ path: 'agents', lazy: () => import('./routes/agent-list') },
				],
			},
		],
	},
] as RouteObject[];

export const router = createBrowserRouter(routes);
