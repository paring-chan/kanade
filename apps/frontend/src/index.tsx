import React from "react";
import ReactDOM from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router";
import routes from "./router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtoolsPanel } from "@tanstack/react-query-devtools";
import { TanStackDevtools } from "@tanstack/react-devtools";
import { formDevtoolsPlugin } from "@tanstack/react-form-devtools";

import "./global.css";
import { Toaster } from "sonner";

const rootEl = document.getElementById("root");
if (rootEl) {
  const router = createBrowserRouter(routes);

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 1000 * 60 * 5,
        gcTime: 1000 * 60 * 10,
        retry: false,
      },
    },
  });

  const root = ReactDOM.createRoot(rootEl);
  root.render(
    <React.StrictMode>
      <Toaster richColors />
      <QueryClientProvider client={queryClient}>
        <TanStackDevtools
          plugins={[
            formDevtoolsPlugin(),
            {
              name: "Tanstack Query",
              render: <ReactQueryDevtoolsPanel />,
            },
          ]}
        />
        <RouterProvider router={router} />
      </QueryClientProvider>
    </React.StrictMode>,
  );
}
