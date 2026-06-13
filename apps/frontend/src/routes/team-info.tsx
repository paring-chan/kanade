import { ProjectItem } from "../components/project-item";
import { CreateProjectDialog } from "../components/dialog/create-project";
import { useParams } from "react-router";
import { useSuspenseQueries } from "@tanstack/react-query";
import { teamBySlugQueryOptions, teamReposQueryOptions } from "../queries/team";

export const Component = () => {
  const params = useParams<"team">();
  const [{ data: team }, { data: repos }] = useSuspenseQueries({
    queries: [
      teamBySlugQueryOptions(params.team!),
      teamReposQueryOptions(params.team!),
    ],
  });

  return (
    <div className="px-4">
      <div className="container mx-auto mt-16">
        <div>팀</div>
        <div className="flex items-end gap-2">
          <h1 className="text-3xl font-bold">{team.name}</h1>
          <span className="text-base opacity-60">{team.slug}</span>
        </div>

        <div className="flex justify-between items-center mt-4">
          <h2 className="text-2xl font-medium">소속 프로젝트</h2>
          <CreateProjectDialog defaultTeamId={team.id} />
        </div>

        <div className="mt-4 grid lg:grid-cols-2">
          {repos.map((repo, i) => (
            <ProjectItem repo={repo} key={i} />
          ))}
        </div>
      </div>
    </div>
  );
};
