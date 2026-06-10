import { tv } from "tailwind-variants";

export const dialog = tv({
  slots: {
    backdrop: [
      "fixed inset-0 min-h-dvh bg-black opacity-20",
      "transition-opacity duration-150 z-50",
      "data-starting-style:opacity-0 data-ending-style:opacity-0",
    ],
    popup: [
      "fixed top-1/2 left-1/2 z-50 -translate-1/2",
      "bg-pink-100 shadow-md outline-black/20 p-4",
      "transition-[opacity,scale]",
      "data-starting-style:opacity-0 data-starting-style:scale-95",
      "data-ending-style:opacity-0 data-ending-style:scale-95",
      "max-w-lg w-full",
      "flex flex-col gap-4",
    ],
    titleArea: "flex flex-col gap-2",
    title: "text-xl font-medium",
    description: "text-base",
    actionsArea: "flex gap-2 justify-end",
    content: "flex flex-col gap-4",
  },
})();
