import { ScrollArea } from "@base-ui/react";
import { Fragment, useRef, useState, useSyncExternalStore } from "react";
import { cn } from "tailwind-variants";
import { useVirtualizer } from "@tanstack/react-virtual";

import LuChevronRight from "~icons/lucide/chevron-right";
import { keepPreviousData } from "@tanstack/react-query";
import type { components } from "../utils/api/types";

type Step = {
  id: string;
  name: string;
};

type StepInfo = {
  step: Step;
  size: number;
  start: number;
};

type StoreMeta = {
  steps: StepInfo[];
  total: number;
};

type ElementInfo =
  | {
      type: "header";
      step: Step;
    }
  | {
      type: "line";
      index: number;
      line: string;
    };

class LogStore {
  steps: Step[];
  currentMeta!: StoreMeta;
  logs = new Map<string, string[]>();
  callbacks = new Set<() => void>();

  constructor() {
    this.steps = [
      {
        id: "step1",
        name: "Step 1",
      },
      {
        id: "step2",
        name: "Step 2",
      },
      {
        id: "step2",
        name: "Step 3",
      },
    ];

    this.logs.set(
      "step1",
      Array.from({ length: 1000 }, (_, i) => `Log ${i}`),
    );

    this.logs.set(
      "step2",
      Array.from({ length: 5000 }, (_, i) => `Log ${i}`),
    );

    this.logs.set(
      "step3",
      Array.from({ length: 2000 }, (_, i) => `Log ${i}`),
    );

    this.rebuildMeta();
  }

  private notify() {
    this.callbacks.forEach((x) => x());
  }

  private rebuildMeta() {
    const meta: StoreMeta = { total: 0, steps: [] };
    for (const step of this.steps) {
      const logCount = this.logs.get(step.id)?.length ?? 0;

      const sizeVisible = 1 + logCount;

      meta.steps.push({
        step,
        size: sizeVisible,
        start: meta.total,
      });
      meta.total += sizeVisible;
    }

    this.currentMeta = meta;

    this.notify();
  }

  currentStep(index: number) {
    let result = this.currentMeta.steps[0];
    for (let i = 1; i < this.currentMeta.steps.length; i++) {
      const current = this.currentMeta.steps[i]!;
      if (current.start <= index) result = current;
      else break;
    }

    return result;
  }

  get(index: number): ElementInfo | null {
    const meta = this.currentStep(index);

    if (!meta) return null;

    if (meta.start === index) return { type: "header", step: meta.step };
    const logs = this.logs.get(meta.step.id)!;

    const logIndex = index - meta.start - 1;
    return { type: "line", line: logs[logIndex]!, index: logIndex };
  }

  subscribe(callback: () => void): () => void {
    this.callbacks.add(callback);

    return () => {
      this.callbacks.delete(callback);
    };
  }
}

export const LogView = () => {
  const container = useRef<HTMLDivElement>(null);

  const [store] = useState(() => new LogStore());
  const meta = useSyncExternalStore(
    (cb) => store.subscribe(cb),
    () => store.currentMeta,
  );

  const virtualizer = useVirtualizer({
    count: meta.total,
    getScrollElement: () => container.current,
    estimateSize: () => 24,
  });

  return (
    <ScrollArea.Root className="flex flex-col size-full">
      <ScrollArea.Viewport className="grow h-0 overflow-y-auto" ref={container}>
        <ScrollArea.Content>
          <div
            style={{ height: `${virtualizer.getTotalSize()}px` }}
            className="relative"
          >
            {virtualizer.getVirtualItems().map((row) => {
              const item = store.get(row.index);
              if (!item) return <Fragment key={row.key} />;

              switch (item.type) {
                case "header":
                  return (
                    <button
                      key={row.key}
                      data-index={row.index}
                      ref={virtualizer.measureElement}
                      className="bg-pink-200 hover:bg-pink-300 cursor-pointer px-2 py-1 w-full text-left flex items-center gap-2 top-0 z-10 absolute"
                      style={{
                        transform: `translateY(${row.start}px)`,
                      }}
                    >
                      <div className="size-4">
                        <LuChevronRight className="size-4 rotate-90" />
                      </div>
                      <div className="text-sm">Step</div>
                    </button>
                  );
                case "line":
                  return (
                    <div
                      key={row.key}
                      data-index={row.index}
                      ref={virtualizer.measureElement}
                      className="px-2 text-sm font-mono py-0.5 hover:bg-black/5 absolute w-full"
                      style={{
                        transform: `translateY(${row.start}px)`,
                      }}
                    >
                      {item.line}
                    </div>
                  );
              }
            })}
          </div>
        </ScrollArea.Content>
      </ScrollArea.Viewport>
      <ScrollArea.Scrollbar
        className={cn(
          "flex w-2 z-30",
          "opacity-0 transition-opacity data-hovering:opacity-100 bg-pink-300/30",
          "data-scrolling:pointer-events-auto",
        )}
      >
        <ScrollArea.Thumb className="w-full bg-pink-300" />
      </ScrollArea.Scrollbar>
    </ScrollArea.Root>
  );
};
