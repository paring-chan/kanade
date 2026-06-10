import { queryOptions } from "@tanstack/react-query";
import { api } from "../utils/api";

export const teamListQueryOptions = () =>
  queryOptions({
    queryKey: ["teams"],
    queryFn: () => api.GET("/api/v1/teams").then((x) => x.data!),
  });

export const teamByIdQueryOptions = (id: string) =>
  queryOptions({
    queryKey: ["teams", "by-id", id],
    queryFn: () =>
      api
        .GET("/api/v1/teams/{team_id}", {
          params: { path: { team_id: id } },
        })
        .then((x) => x.data!),
  });

export const teamBySlugQueryOptions = (slug: string) =>
  queryOptions({
    queryKey: ["teams", "by-slug", slug],
    queryFn: () =>
      api
        .GET("/api/v1/teams/by-slug/{team_slug}", {
          params: { path: { team_slug: slug } },
        })
        .then((x) => x.data!),
  });
