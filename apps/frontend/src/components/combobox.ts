import { tv } from 'tailwind-variants';

export const combobox = tv({
	slots: {
		inputGroup: 'outline outline-black/20 flex h-8',
		input: 'w-0 grow outline-none px-2 py-1',
		trigger: 'flex items-center justify-center size-8',
		positioner: 'z-50',
		popup: [
			'w-(--anchor-width) max-w-(--available-width)',
			'max-h-(--available-height) transition-[opacity,scale]',
			'data-starting-style:opacity-0 data-starting-style:scale-95',
			'data-ending-style:opacity-0 data-ending-style:scale-95',
			'bg-pink-50 shadow-lg overflow-y-scroll',
		],
		empty: 'text-center p-2',

		item: ['p-2 select-none not-disabled:cursor-pointer transition-colors truncate', 'data-highlighted:bg-pink-100 grid grid-cols-[2rem_1fr]', 'data-selected:bg-pink-200'],
		itemIndicator: 'col-start-1 justify-center flex pt-0.5',
		itemContent: 'col-start-2',
	},
})();
