import {
  maybeParseWorkerMessage,
  type SvgExportRequest,
  type SvgExportWorkerMessage,
} from "./messages";

export type SvgExportHandlers = {
  readonly onSampling: (completed: number, total: number) => void;
  readonly onAssembling: (total: number) => void;
  readonly onComplete: (result: {
    readonly svg: string;
    readonly elapsedMs: number;
  }) => void;
  readonly onError: (message: string) => void;
};

export type SvgExportClient = {
  start(request: SvgExportRequest, handlers: SvgExportHandlers): void;
  cancel(): void;
};

/** Owns one dedicated worker so SVG sampling never steps the visible scene. */
export function createSvgExportClient(): SvgExportClient {
  let maybeWorker: Worker | undefined;
  let job = 0;

  function cancel(): void {
    job += 1;
    const worker = maybeWorker;
    maybeWorker = undefined;
    if (worker === undefined) {
      return;
    }

    worker.onmessage = null;
    worker.onerror = null;
    worker.terminate();
  }

  return {
    cancel,
    start(request, handlers) {
      cancel();
      const currentJob = job;
      const worker = new Worker(new URL("./svg-export-worker.ts", import.meta.url), {
        type: "module",
      });
      maybeWorker = worker;
      worker.onmessage = (event: MessageEvent<unknown>) => {
        if (currentJob !== job) {
          return;
        }

        deliver(event.data, handlers);
      };
      worker.onerror = () => {
        if (currentJob !== job) {
          return;
        }

        handlers.onError("The SVG export worker failed.");
      };
      worker.postMessage(request);
    },
  };
}

function deliver(data: unknown, handlers: SvgExportHandlers): void {
  const maybeMessage = maybeParseWorkerMessage(data);
  if (maybeMessage === undefined) {
    handlers.onError("The SVG export worker sent an invalid message.");
    return;
  }

  dispatch(maybeMessage, handlers);
}

function dispatch(message: SvgExportWorkerMessage, handlers: SvgExportHandlers): void {
  switch (message.type) {
    case "sampling":
      handlers.onSampling(message.completed, message.total);
      return;
    case "assembling":
      handlers.onAssembling(message.total);
      return;
    case "complete":
      handlers.onComplete({ svg: message.svg, elapsedMs: message.elapsedMs });
      return;
    case "error":
      handlers.onError(message.message);
      return;
  }
}
