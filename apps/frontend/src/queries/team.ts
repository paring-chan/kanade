import { queryOptions } from "@tanstack/react-query";
import { api } from "../utils/api";

export const teamListQueryOptions = () =>
  queryOptions({
    queryKey: ["teams"],
    queryFn: () => api.GET("/api/v1/teams").then((x) => x.data!),
  });
