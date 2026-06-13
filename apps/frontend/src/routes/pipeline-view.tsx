import X from '~icons/lucide/x';

export const Component = () => {
	return (
		<div className="flex grow">
			<div className="grow w-0 flex flex-col">
				<div className="flex w-full relative h-8">
					<div className="absolute inset-0 pointer-events-none border-b  border-black/10"></div>
					<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">파이프라인 정보</button>
					<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors border-b  border-black">트리</button>
					<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">DAG</button>
				</div>
			</div>
			<div className="grow w-0 border-l border-black/10 flex flex-col">
				<div className="flex w-full relative h-8">
					<div className="absolute inset-0 pointer-events-none border-b  border-black/10"></div>
					<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors border-b  border-black">로그</button>
					<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">아티팩트</button>
					<button className="flex items-center cursor-pointer px-3 hover:bg-black/10 transition-colors">상세정보</button>
					<div className="grow" />
					<button className="h-8 w-8 flex justify-center items-center cursor-pointer hover:bg-black/10 transition-colors">
						<X className="size-4" />
					</button>
				</div>
				<div className="bg-white grow">대충로그뷰</div>
			</div>
		</div>
	);
};
