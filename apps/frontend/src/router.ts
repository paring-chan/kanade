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
					{ path: 'repo/:team/:repo/pipelines/:pipeline', lazy: () => import('./routes/pipeline-view') },
					{ path: 'login/success', lazy: () => import('./routes/login-success') },
					{ path: 'teams', lazy: () => import('./routes/team-list') },
					{ path: 't/:team', lazy: () => import('./routes/team-info') },
					{ path: 'r/:team/:repo', lazy: () => import('./routes/repo-info') },
				],
			},
		],
	},
] as RouteObject[];

export const router = createBrowserRouter(routes);
