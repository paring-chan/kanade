import { tv } from 'tailwind-variants';

export const button = tv({
  base: 'px-4 py-1 cursor-pointer transition-colors',
  variants: {
    style: {
      normal: 'bg-black text-white',
      outlined: 'border border-black/40 hover:bg-black/10 active:bg-black/15',
    },
  },
});
