import { Button } from '@base-ui/react';
import { button } from '../components/button';
import { ProjectItem } from '../components/project-item';

export const Component = () => {
	return (
		<div className="px-4">
			<div className="container mx-auto mt-12">
				<div className="flex items-center">
					<h1 className="text-3xl grow w-0">프로젝트 목록</h1>
					<Button className={button({ style: 'outlined' })}>생성</Button>
				</div>

				<div className="mt-4 grid lg:grid-cols-2">
					{/*{Array.from({ length: 30 }).map((_, i) => (
            <ProjectItem key={i} />
          ))}*/}
				</div>

				<div className="text-center p-4">-END-</div>
			</div>
		</div>
	);
};
