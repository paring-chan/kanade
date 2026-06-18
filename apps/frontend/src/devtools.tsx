import { ReactQueryDevtoolsPanel } from "@tanstack/react-query-devtools";
import { TanStackDevtools } from "@tanstack/react-devtools";
import { formDevtoolsPlugin } from "@tanstack/react-form-devtools";

export const Devtools = () => {
  return (
    <TanStackDevtools
      plugins={[
        formDevtoolsPlugin(),
        {
          name: "Tanstack Query",
          render: <ReactQueryDevtoolsPanel />,
        },
      ]}
    />
  );
};
