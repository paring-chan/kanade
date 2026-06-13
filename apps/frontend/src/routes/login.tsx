import { useLoaderData } from 'react-router';
import { api } from '../utils/api';

export const loader = async () => {
	const { data } = await api.GET('/api/v1/forges');

	return {
		forges: data!,
	};
};

export const Component = () => {
	const { forges } = useLoaderData<typeof loader>();

	return (
		<div className="flex justify-center mt-16">
			<div className="max-w-96 w-full">
				<h1 className="text-3xl text-center">로그인</h1>
				<p className="text-center mt-2">로그인에 사용할 서버를 선택하세요</p>
				<div className="flex flex-col gap-2 mt-4">
					{forges.map((x) => (
						<a key={x.id} className="border border-black/10 hover:border-black transition-colors px-4 py-2 text-center rounded-lg" href={'/_/auth/login/' + x.id}>
							{x.name}
						</a>
					))}
				</div>
			</div>
		</div>
	);
};
