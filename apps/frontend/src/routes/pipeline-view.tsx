import {
  generatePath,
  Link,
  useParams,
  type LoaderFunction,
} from "react-router";
import { queryClient } from "../utils/api";
import {
  pipelineJobsQueryOptions,
  pipelineQueryOptions,
} from "../queries/pipeline";
import { repoByIdQueryOptions } from "../queries/repo";

import { Suspense } from "react";
import { useSuspenseQuery } from "@tanstack/react-query";
import type { components } from "../utils/api/types";

import LuX from "~icons/lucide/x";
import LuLoaderCircle from "~icons/lucide/loader-circle";
import LuHourglass from "~icons/lucide/hourglass";
import LuBan from "~icons/lucide/ban";
import LuChevronsRight from "~icons/lucide/chevrons-right";
import LuCheck from "~icons/lucide/check";

export const loader = (async ({ params }) => {
  const pipeline = await queryClient.ensureQueryData(
    pipelineQueryOptions(params.pipeline!),
  );
  const repo = await queryClient.ensureQueryData(
    repoByIdQueryOptions(pipeline.repoId),
  );

  return { pipeline, repo };
}) satisfies LoaderFunction;

const Breadcrumb = ({
  pipeline: { id: pipelineId },
  repo,
}: Awaited<ReturnType<typeof loader>>) => {
  const { data: pipeline } = useSuspenseQuery(pipelineQueryOptions(pipelineId));

  const icon = (() => {
    switch (pipeline.status) {
      case "cancelled":
        return <LuBan className="size-4 text-gray-400" />;
      case "queued":
        return <LuHourglass className="size-4 text-yellow-400" />;
      case "running":
        return (
          <LuLoaderCircle className="size-4 animate-spin text-yellow-400" />
        );
      case "failed":
        return <LuX className="size-4 text-red-500" />;
      case "success":
        return <LuCheck className="size-4 text-green-500" />;
    }
  })();

  return (
    <div className="flex gap-1 items-center">
      <Link
        to={generatePath("/t/:team", { team: repo.team.slug })}
        className="hover:underline"
      >
        {repo.team.name}
      </Link>
      <span> / </span>
      <Link
        to={generatePath("/r/:team/:repo", {
          team: repo.team.slug,
          repo: repo.slug,
        })}
        className="hover:underline"
      >
        {repo.name}
      </Link>
      <span> / </span>
      <Link
        to={generatePath("/p/:pipeline", {
          pipeline: pipeline.id,
        })}
        className="hover:underline"
      >
        #{pipeline.serial}
      </Link>

      <div className="border-b w-2 mx-2 border-black/20" />

      <div className="font-semibold">{pipeline.title}</div>

      <div className="ml-2">{icon}</div>
    </div>
  );
};

export const handle = {
  Breadcrumb,
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
          {/*<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">
            파이프라인 정보
          </button>*/}
          <button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors border-b  border-black">
            작업 목록
          </button>
          {/*<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">
            DAG
          </button>*/}
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
            <LuX className="size-4" />
          </button>
        </div>
        <div className="bg-white grow overflow-y-auto">대충로그뷰</div>
      </div>
    </div>
  );
};

const JobTree = ({ pipelineId }: { pipelineId: string }) => {
  const { data: job } = useSuspenseQuery(pipelineJobsQueryOptions(pipelineId));

  return (
    <div className="flex flex-col">
      {job.map((job) => (
        <JobItem key={job.id} job={job} />
      ))}
    </div>
  );
};

const JobItem = ({
  job,
}: {
  job: components["schemas"]["PipelineJobResponse"];
}) => {
  const icon = (() => {
    switch (job.status) {
      case "cancelled":
        return <LuBan className="size-4 text-gray-400" />;
      case "pending":
        return <LuHourglass className="size-4 text-yellow-400" />;
      case "waiting":
        return <LuHourglass className="size-4 text-gray-400" />;
      case "running":
        return (
          <LuLoaderCircle className="size-4 animate-spin text-yellow-400" />
        );
      case "skipped":
        return <LuChevronsRight className="size-4 text-gray-400" />;
      case "failed":
        return <LuX className="size-4 text-red-500" />;
      case "success":
        return <LuCheck className="size-4 text-green-500" />;
    }
  })();

  return (
    <button className="px-4 py-1 flex items-center gap-2 transition-colors text-left cursor-pointer hover:bg-black/5">
      <div className="size-4">{icon}</div>
      <div className="grow">{job.name}</div>
    </button>
  );
};
