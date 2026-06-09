import { tv } from 'tailwind-variants';

const menuStyles = tv({
  slots: {
    positioner: 'z-50',
    popup: [
      'relative origin-(--transform-origin) bg-pink-100 border border-black/10 min-w-32 text-sm',
      'flex flex-col',
      'transition-[scale,opacity]',
      'data-starting-style:scale-[0.98] data-starting-style:opacity-0',
      'data-ending-style:scale-[0.98] data-ending-style:opacity-0',
    ],

    item: 'px-2 py-1 hover:bg-black/5 transition-colors text-left cursor-pointer',
  },
});

export const menu = menuStyles();
