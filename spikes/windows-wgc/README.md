# Persistent Windows Graphics Capture spike

Uses `windows-capture` 2.0 over Windows Graphics Capture. Each operating mode creates one persistent capture session and consumes 100 frames. Captured pixels remain in memory.

Modes:

- `full`: full 2880x1800 GPU-to-CPU readback.
- `center_region`: a centered region one third of each screen dimension, suitable for region-based Smart Scroll preprocessing.

```powershell
powershell -ExecutionPolicy Bypass -File spikes/windows-wgc/run.ps1
```

The WGC frame pool/session remains persistent. The wrapper reuses capture-session GPU resources, but its current CPU `buffer`/`buffer_crop` methods allocate a staging texture per call; that limitation is recorded rather than hidden.

WGC is change-driven. `run.ps1` launches a deterministic local WinForms animation at approximately 60 Hz so the 100-frame reliability and interval measurements do not depend on unrelated desktop activity.
