import { tv } from "tailwind-variants";

export const formField = tv({
  slots: {
    root: "flex flex-col gap-2",
    label: "text-base font-medium",

    helperArea: "flex flex-col",
    description: "text-sm opacity-60",
    error: "text-xs text-red-500",
  },
})();

export const input = tv({
  base: "px-2 py-1",
  variants: {
    style: {
      outlined: [
        "outline outline-black/20 focus:outline-black ring-blue-400 transition-colors",
      ],
    },
  },
  defaultVariants: {
    style: "outlined",
  },
});
