import { Link } from 'react-router';
import GitCommitHorizontal from '~icons/lucide/git-commit-horizontal';
import type { components } from '../utils/api/types';

export const ProjectItem = ({ repo }: { repo: components['schemas']['RepoResponse'] }) => {
	return (
		<div className="p-4 flex items-center gap-4 bg-pink-50 border border-black/10 -ml-px -mt-px">
			<div className="max-w-64 grow min-w-0">
				<div className="text-lg leading-3.5 flex whitespace-nowrap truncate pb-1">
					<Link className="hover:underline" to="/teams/mizuki">
						{repo.team.slug}
					</Link>
					<span className="px-1">/</span>
					<Link className="hover:underline" to="/repo/mizuki/kurukuru">
						{repo.slug}
					</Link>
				</div>
				<a href="https://git.pari.ng/mizuki/kurukuru" className="text-xs opacity-40 hover:underline truncate" target="_blank" rel="noopener">
					git.pari.ng/mizuki/kurukuru
				</a>
			</div>

			<div className="grow text-right">
				<Link to="/repo/mizuki/kurukuru/pipelines/123" className="max-w-14 w-full hover:underline">
					#123
				</Link>
				<div className="grow truncate text-sm flex items-center justify-end">
					<span>이것은 샘플입니다.</span>
					<GitCommitHorizontal className="ml-2" />
					<a href="https://git.pari.ng/repo/mizuki/commit/xxxxxxxxxxxxxxxxxxxxx" className="hover:underline" target="_blank" rel="noopener">
						a1b2c3d4
					</a>
				</div>
			</div>
		</div>
	);
};
