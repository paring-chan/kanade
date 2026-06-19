import { ScrollArea } from "@base-ui/react";
import {
  Fragment,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { cn } from "tailwind-variants";
import { useVirtualizer } from "@tanstack/react-virtual";

import LuChevronRight from "~icons/lucide/chevron-right";
import type { LogEntry } from "../ws-types";
import { useSuspenseQuery } from "@tanstack/react-query";
import { pipelineJobsQueryOptions } from "../queries/pipeline";
import type { components } from "../utils/api/types";
import { api } from "../utils/api";

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
  shouldConnect = false;

  steps!: Step[];
  currentMeta!: StoreMeta;
  logs = new Map<string, string[]>();
  callbacks = new Set<() => void>();

  private ws?: WebSocket;

  constructor(public job: components["schemas"]["PipelineJobResponse"]) {
    this.setJob(job);
  }

  setJob(job: components["schemas"]["PipelineJobResponse"]) {
    this.job = job;
    this.logs.clear();
    this.steps = job.steps.map((x) => ({ id: x.id, name: x.name }));

    api
      .GET("/api/v1/jobs/{job_id}/logs", {
        params: { path: { job_id: job.id } },
      })
      .then((x) => {
        if (job !== this.job) return;
        const data = x.data!;

        this.logs = new Map();

        for (const log of data) {
          this.append(log.stepId, log.content);
        }

        this.rebuildMeta();
      });

    this.rebuildMeta();
  }

  append(stepId: string, message: string) {
    let logs = this.logs.get(stepId);
    if (!logs) {
      logs = [];
      this.logs.set(stepId, logs);
    }

    const lines = message.split("\n");

    lines.forEach((line, i) => {
      if (i > 0) logs.push(line);
      else {
        if (logs.length === 0) {
          logs.push(message);
          return;
        }
        logs[logs.length - 1] += line;
      }
    });

    this.rebuildMeta();
  }

  connect() {
    if (this.ws) {
      this.ws.close();
    }

    const loc = window.location;
    const ws = new WebSocket(
      `${loc.protocol.replace("http", "ws")}//${loc.host}/_/ws/logs/${this.job.id}`,
    );
    this.ws = ws;
    this.shouldConnect = true;

    ws.onopen = () => {
      console.log("ws connected");
    };

    ws.onmessage = (ev) => {
      try {
        const data = JSON.parse(ev.data) as LogEntry;

        this.append(data.stepId, data.content);
      } catch (e) {
        console.error("failed to parse message:", ev.data, e);
      }
    };

    ws.onclose = () => {
      console.log("connection closed");

      if (!this.shouldConnect) return;
      setTimeout(() => {
        if (!this.shouldConnect) return;
        console.log("reconnecting...");
        this.connect();
      }, 1000);
    };
  }

  disconnect() {
    this.shouldConnect = false;
    this.ws?.close();
  }

  private notify() {
    this.callbacks.forEach((x) => {
      x();
    });
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

export const LogView = ({
  jobId,
  pipelineId,
}: {
  pipelineId: string;
  jobId: string;
}) => {
  const { data: jobs } = useSuspenseQuery(pipelineJobsQueryOptions(pipelineId));
  const job = jobs.find((x) => x.id === jobId);
  if (!job) throw new Error("job not found");

  const container = useRef<HTMLDivElement>(null);

  const [store] = useState(() => new LogStore(job));

  useEffect(() => {
    if (store.job.id !== job.id) {
      store.setJob(job);
    }
  }, [job, store]);

  useEffect(() => {
    store.connect();
    return () => {
      store.disconnect();
    };
  }, [store]);

  const meta = useSyncExternalStore(
    (cb) => store.subscribe(cb),
    () => store.currentMeta,
  );

  const virtualizer = useVirtualizer({
    count: meta.total,
    getScrollElement: () => container.current,
    estimateSize: () => 24,
  });

  const totalSize = virtualizer.getTotalSize();

  useEffect(() => {
    if (container.current) {
      container.current.scrollTo({ top: totalSize, behavior: "smooth" });
    }
  }, [totalSize]);

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
                      <div className="text-sm">{item.step.name}</div>
                    </button>
                  );
                case "line":
                  return (
                    <div
                      key={row.key}
                      data-index={row.index}
                      className="flex gap-2 absolute px-2 text-sm font-mono py-0.5 hover:bg-black/5 w-full"
                      style={{
                        transform: `translateY(${row.start}px)`,
                      }}
                    >
                      <div className="opacity-40">
                        {(item.index + 1).toString().padStart(6, "0")}
                      </div>
                      <pre ref={virtualizer.measureElement}>{item.line}</pre>
                    </div>
                  );
                default:
                  throw new Error("what");
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
