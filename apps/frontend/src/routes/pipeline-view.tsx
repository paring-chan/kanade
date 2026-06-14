import {
  generatePath,
  Link,
  useParams,
  type LoaderFunction,
} from "react-router";
import X from "~icons/lucide/x";
import { queryClient } from "../utils/api";
import {
  pipelineJobsQueryOptions,
  pipelineQueryOptions,
} from "../queries/pipeline";
import { repoByIdQueryOptions } from "../queries/repo";

import LuLoaderCircle from "~icons/lucide/loader-circle";
import { Suspense } from "react";
import { useSuspenseQuery } from "@tanstack/react-query";

export const loader = (async ({ params }) => {
  const pipeline = await queryClient.ensureQueryData(
    pipelineQueryOptions(params.pipeline!),
  );
  const repo = await queryClient.ensureQueryData(
    repoByIdQueryOptions(pipeline.repoId),
  );

  return { pipeline, repo };
}) satisfies LoaderFunction;

export const handle = {
  breadcrumb: (data: Awaited<ReturnType<typeof loader>>) => {
    return (
      <div className="flex gap-1 items-center">
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
        <span> / </span>
        <Link
          to={generatePath("/p/:pipeline", {
            pipeline: data.pipeline.id,
          })}
          className="hover:underline"
        >
          #{data.pipeline.serial}
        </Link>

        <div className="border-b w-2 mx-2 border-black/20" />

        <div className="font-semibold">{data.pipeline.title}</div>
      </div>
    );
  },
};

export const Component = () => {
  const params = useParams<"pipeline">();

  const { data: pipeline } = useSuspenseQuery(
    pipelineQueryOptions(params.pipeline!),
  );

  return (
    <div className="flex grow">
      <div className="grow w-0 flex flex-col">
        <div className="flex w-full relative h-8">
          <div className="absolute inset-0 pointer-events-none border-b  border-black/10"></div>
          <button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">
            파이프라인 정보
          </button>
          <button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors border-b  border-black">
            트리
          </button>
          <button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">
            DAG
          </button>
        </div>

        {/*content*/}
        <div className="grow overflow-y-auto h-0">
          <Suspense
            fallback={
              <div className="flex justify-center p-8">
                <LuLoaderCircle className="animate-spin" />
              </div>
            }
          >
            <JobTree pipelineId={pipeline.id} />
          </Suspense>
        </div>
      </div>
      <div className="grow w-0 border-l border-black/10 flex flex-col">
        <div className="flex w-full relative h-8">
          <div className="absolute inset-0 pointer-events-none border-b  border-black/10"></div>
          <button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors border-b  border-black">
            로그
          </button>
          <button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">
            아티팩트
          </button>
          <button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">
            상세정보
          </button>
          <div className="grow" />
          <button className="h-8 w-8 flex justify-center items-center cursor-pointer hover:bg-black/10 transition-colors">
            <X className="size-4" />
          </button>
        </div>
        <div className="bg-white grow overflow-y-auto">대충로그뷰</div>
      </div>
    </div>
  );
};

const JobTree = ({ pipelineId }: { pipelineId: string }) => {
  const { data } = useSuspenseQuery(pipelineJobsQueryOptions(pipelineId));

  return <pre>{JSON.stringify(data, null, 2)}</pre>;
};
