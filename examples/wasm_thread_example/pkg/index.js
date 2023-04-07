const worker = new Worker("./worker.js", { type: "module" });

worker.postMessage({ type: "init" });
worker.onmessage = (event) => {
  if (event.data.type !== "initResponse") return;

  for (let i = 0; i < 100000; i++) {
    worker.postMessage({ type: "encode" });
  }

  worker.postMessage({ type: "finish" });
};
