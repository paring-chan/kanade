import { useSuspenseQuery } from "@tanstack/react-query";
import { repoQueryOptions } from "../queries/repo";
import {
  generatePath,
  Link,
  useParams,
  type LoaderFunction,
} from "react-router";
import { queryClient } from "../utils/api";

export const loader = (async ({ params }) => {
  const repo = await queryClient.ensureQueryData(
    repoQueryOptions(params.team!, params.repo!),
  );

  return {
    repo,
  };
}) satisfies LoaderFunction;

export const handle = {
  breadcrumb: (data: Awaited<ReturnType<typeof loader>>) => (
    <div>
      <Link
        to={generatePath("/t/:team", { team: data.repo.team.slug })}
        className="hover:underline"
      >
        {data.repo.team.name}
      </Link>
      <span> / </span>
      <Link
        to={generatePath("/r/:team/:repo", {
          team: data.repo.team.slug,
          repo: data.repo.slug,
        })}
        className="hover:underline"
      >
        {data.repo.name}
      </Link>
    </div>
  ),
};

export const Component = () => {
  const params = useParams<"team" | "repo">();
  const { data: repo } = useSuspenseQuery(
    repoQueryOptions(params.team!, params.repo!),
  );

  return <pre>{JSON.stringify(repo, null, 2)}</pre>;
};
