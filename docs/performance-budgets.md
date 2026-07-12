# Performance budgets

These are release gates measured on named reference hardware, not universal claims.

| Metric | Budget | Method |
|---|---:|---|
| Tray-idle CPU | <= 0.5% mean, <= 1.0% p95 | 1 Hz process-tree deltas after 10 s warm-up |
| Tray-idle private memory | <= 60 MiB p95 | Sum private working set for process tree |
| Hidden-webview private memory | Record separately | Same method; not treated as tray-idle target |
| Primary-monitor capture | <= 35 ms median, <= 60 ms p95 | 100 release captures after 10 warm-ups |
| Clear multi-QR decode | <= 40 ms median, <= 100 ms p95 | 100 release decodes, exact results required |
| Webcam steady sampling | <= 10 decode FPS by default | Capture may be faster; decoder queue is bounded |
| Idle capture/decoder work | Zero calls | Integration counters/assertions |

## Runtime policy

- No polling while idle.
- One decode in flight and at most one replaceable pending frame.
- Reuse buffers; do not encode images in the scan path.
- Smart Scroll performs a cheap frame-change test before decode and backs off on unchanged content.
- Webcam frames are dropped under backpressure.
- Regressions over 15% require explanation; budget violations block release.

