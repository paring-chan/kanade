import type { RouteObject } from 'react-router';

export default [
  {
    lazy: () => import('./layouts/root'),
    children: [
      {
        lazy: () => import('./layouts/normal'),
        children: [
          {
            index: true,
            lazy: () => import('./routes/project-list'),
          },
        ],
      },
    ],
  },
] as RouteObject[];
