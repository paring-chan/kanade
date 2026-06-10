import { queryOptions } from "@tanstack/react-query";
import { api } from "../utils/api";
import { HTTPError } from "ky";

export const userQueryOptions = () =>
  queryOptions({
    queryKey: ["users", "me"],
    queryFn: () =>
      api
        .GET("/api/v1/users/me")
        .then((x) => x.data)
        .catch((e) => {
          if (e instanceof HTTPError && e.response.status === 401) return null;

          return Promise.reject(e);
        }),
  });

export const userForgesQueryOptions = () =>
  queryOptions({
    queryKey: ["users", "me", "forges"],
    queryFn: () => api.GET("/api/v1/users/me/forges").then((x) => x.data!),
  });
