import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router';
import { router } from './router';
import { QueryClientProvider } from '@tanstack/react-query';

import './global.css';
import { Toaster } from 'sonner';
import { queryClient } from './utils/api';
import './utils/ws';

const DevTools = import.meta.env.DEV ? await import('./devtools').then((x) => x.Devtools) : () => null;

const rootEl = document.getElementById('root');
if (rootEl) {
	const root = ReactDOM.createRoot(rootEl);
	root.render(
		<React.StrictMode>
			<Toaster richColors />
			<QueryClientProvider client={queryClient}>
				<DevTools />
				<RouterProvider router={router} />
			</QueryClientProvider>
		</React.StrictMode>,
	);
}
