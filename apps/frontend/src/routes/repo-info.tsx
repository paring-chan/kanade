import { useSuspenseQuery } from "@tanstack/react-query";
import { repoQueryOptions } from "../queries/repo";
import { useParams } from "react-router";

export const Component = () => {
  const params = useParams<"team" | "repo">();
  const { data: repo } = useSuspenseQuery(
    repoQueryOptions(params.team!, params.repo!),
  );

  return <pre>{JSON.stringify(repo, null, 2)}</pre>;
};
