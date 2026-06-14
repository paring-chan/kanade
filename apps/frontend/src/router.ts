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

					{ path: 'p/:pipeline', lazy: () => import('./routes/pipeline-view') },
				],
			},
		],
	},
] as RouteObject[];

export const router = createBrowserRouter(routes);
