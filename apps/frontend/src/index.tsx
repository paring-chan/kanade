import React from 'react';
import ReactDOM from 'react-dom/client';
import { createBrowserRouter, RouterProvider } from 'react-router';
import routes from './router';
import './global.css';

const rootEl = document.getElementById('root');
if (rootEl) {
  const router = createBrowserRouter(routes);

  const root = ReactDOM.createRoot(rootEl);
  root.render(
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>,
  );
}
