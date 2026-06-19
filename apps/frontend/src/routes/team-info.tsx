import { ProjectItem } from '../components/project-item';
import { CreateProjectDialog } from '../components/dialog/create-project';
import { generatePath, Link, useParams, type LoaderFunction } from 'react-router';
import { useSuspenseQueries } from '@tanstack/react-query';
import { teamBySlugQueryOptions, teamReposQueryOptions } from '../queries/team';
import { queryClient } from '../utils/api';
import { button } from '../components';

export const loader = (async ({ params }) => {
	const team = await queryClient.ensureQueryData(teamBySlugQueryOptions(params.team!));

	return {
		team,
	};
}) satisfies LoaderFunction;

export const handle = {
	breadcrumb: (data: Awaited<ReturnType<typeof loader>>) => (
		<div>
			<Link to={generatePath('/t/:team', { team: data.team.slug })} className="hover:underline">
				{data.team.name}
			</Link>
		</div>
	),
};

export const Component = () => {
	const params = useParams<'team'>();
	const [{ data: team }, { data: repos }] = useSuspenseQueries({
		queries: [teamBySlugQueryOptions(params.team!), teamReposQueryOptions(params.team!)],
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
					<div className="flex gap-4">
						<Link to={generatePath('/t/:team/secrets', { team: team.slug })} className={button({ style: 'outlined' })}>
							시크릿 관리
						</Link>
						<CreateProjectDialog defaultTeamId={team.id} />
					</div>
				</div>

				<div className="mt-4 grid lg:grid-cols-2">
					{repos.map((repo) => (
						<ProjectItem repo={repo} key={repo.id} />
					))}
				</div>
			</div>
		</div>
	);
};
