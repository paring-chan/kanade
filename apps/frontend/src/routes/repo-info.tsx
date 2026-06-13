import { useSuspenseQuery } from "@tanstack/react-query";
import { repoQueryOptions } from "../queries/repo";
import {
  generatePath,
  Link,
  useParams,
  type LoaderFunction,
} from "react-router";
import { api, queryClient } from "../utils/api";
import type { components } from "../utils/api/types";
import { Suspense } from "react";
import { Button } from "@base-ui/react";

import LuLoaderCircle from "~icons/lucide/loader-circle";
import LuHourglass from "~icons/lucide/hourglass";
import LuX from "~icons/lucide/x";
import LuRefreshCw from "~icons/lucide/refresh-cw";

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

  return (
    <main className="px-4 py-16">
      <div className="container mx-auto">
        <div className="flex justify-between items-center">
          <div>
            <h4 className="text-sm opacity-60">{repo.slug}</h4>
            <h1 className="text-3xl font-semibold">{repo.name}</h1>
          </div>
          <div>asdf</div>
        </div>

        <h2 className="mt-8 text-2xl font-medium">파이프라인</h2>
        <div>
          <Suspense
            fallback={
              <div className="flex justify-center py-4">
                <LuLoaderCircle className="animate-spin size-6" />
              </div>
            }
          >
            <PipelineList repo={repo} />
          </Suspense>
        </div>
      </div>
    </main>
  );
};

const PipelineList = ({
  repo,
}: {
  repo: components["schemas"]["RepoResponse"];
}) => {
  const { data } = useSuspenseQuery({
    queryKey: ["repos", repo.team.slug, repo.slug, "pipelines"],
    queryFn: () =>
      api
        .GET("/api/v1/repos/{team}/{repo}/pipelines", {
          params: {
            path: {
              repo: repo.slug,
              team: repo.team.slug,
            },
          },
        })
        .then((x) => x.data!),
  });

  return (
    <div className="border border-black/10 divide-y divide-black/10 mt-4">
      {data.items.length === 0 && (
        <div className="text-center p-4"> -비어있음- </div>
      )}
      {data.items.map((x) => (
        <PipelineItem pipeline={x} key={x.id} />
      ))}
    </div>
  );
};

const PipelineItem = ({
  pipeline,
}: {
  pipeline: components["schemas"]["PipelineResponse"];
}) => {
  const pipelineLink = generatePath("/pipelines/:id", { id: pipeline.id });

  return (
    <div className="py-4 px-6 gap-4 flex items-center">
      <div className="size-4">
        <LuHourglass className="size-4 text-yellow-400" />
      </div>
      <Link
        to={pipelineLink}
        className="font-mono font-bold hover:underline opacity-60"
      >
        #{pipeline.serial.toString().padStart(6, "0")}
      </Link>
      <Link to={pipelineLink} className="font-medium hover:underline">
        {pipeline.title || "제목 없음"}
      </Link>
      <div className="grow"></div>
      <div>{pipeline.triggeredBy}</div>
      <Button className="size-4 opacity-40 cursor-pointer hover:opacity-100 transition-opacity">
        <LuRefreshCw className="size-4" />
      </Button>
      <Button className="size-4 opacity-60 text-red-400 cursor-pointer hover:opacity-100 transition-opacity">
        <LuX className="size-4" />
      </Button>
    </div>
  );
};
