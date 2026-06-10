import type { RouteObject } from "react-router";

export default [
  {
    lazy: () => import("./layouts/root"),
    children: [
      {
        lazy: () => import("./layouts/normal"),
        children: [
          {
            index: true,
            lazy: () => import("./routes/project-list"),
          },
          {
            path: "repo/:team/:repo/pipelines/:pipeline",
            lazy: () => import("./routes/pipeline-view"),
          },
          {
            path: "login",
            lazy: () => import("./routes/login"),
          },
          {
            path: "login/success",
            lazy: () => import("./routes/login-success"),
          },

          {
            path: "teams",
            lazy: () => import("./routes/team-list"),
          },
          {
            path: "t/:team",
            lazy: () => import("./routes/team-info"),
          },
        ],
      },
    ],
  },
] as RouteObject[];
