nvproton: NVIDIA-tuned Proton stack
High-level goals

Reduce DX12→Vulkan overhead on NVIDIA (better batching, descriptor alloc, pipeline/cache strategy).

Exploit NVIDIA Vulkan extensions when present (low-latency, global priority, present control, RT).

Stabilize frametimes via better submit/present pacing (gamescope/Wayland explicit sync).

Zero-drama deployment: drop-in Proton “compatibility tool” with auto-profiles.

Components & repos
nvproton/
├─ proton/                 # Fork of Valve Proton (wine, protonfixes, build scripts)
├─ vkd3d-proton-nv/        # Fork of vkd3d-proton with NV paths/flags
├─ dxvk-nv/                # DXVK fork: cache/layout tweaks, metrics hooks
├─ nvext/                  # Tiny lib: NVIDIA extension & driver feature probe
├─ nvshim/                 # (optional) LD_PRELOAD Vulkan shim for timing/markers
├─ tools/cachepack/        # Shader/pipeline cache packer/merger + prewarm
├─ tools/profiles/         # Per-title profile DB (YAML) + feature gates
├─ dist/                   # Build artifacts, Proton tool wrapper, version manifest
└─ ci/                     # CI scripts (container builds, checksum, signing)


License sanity: keep upstream licenses intact (Proton: multiple; vkd3d-proton: LGPLv2.1+; DXVK: zlib), publish your patches, and ship source for LGPL parts you modify.

NVIDIA-aware feature layer (nvext)

A tiny C/Zig lib used by vkd3d-proton-nv & dxvk-nv at runtime:

Query & expose:

VK_EXT_global_priority

VK_KHR_present_wait, VK_KHR_present_id

VK_NV_low_latency2 (and older NV low-latency paths)

VK_EXT_frame_boundary

VK_EXT_hdr_metadata

VK_KHR_synchronization2

VK_NV_ray_tracing / VK_KHR_ray_tracing_pipeline

Provide helper decisions:

Which present mode to prefer (mailbox vs fifo-relaxed) under gamescope.

Whether to set global priority HIGH for the graphics queue.

Whether to enable latency markers and present-wait heuristics.

Expose as a thin header + dyn-query functions so it’s easy to call from C/C++/Zig.

vkd3d-proton-nv: DX12 path tuning

Descriptor heap allocator modes:

Add NV_HEAP_STRICT=1|0 and NV_HEAP_BLOCK=256K|1M envs to experiment with fewer, bigger blocks (reduces CPU).

PSO hashing & cache:

Add a multi-segment PSO hash with an “ignore volatile state” option for titles known to churn state objects.

Externalize pipeline cache to NV_PIPELINE_CACHE=/fastcache/$GAMEID.

Barrier elision heuristics (opt-in):

Title-gated flag to coalesce redundant barriers on common engines (UE4/5, RE Engine) when safe.

Async compile queue:

A small background worker pool with bounded latency budget; report spikes via nvshim markers.

dxvk-nv: DX11 path & shared knobs

State cache: coalesce and relocate caches to fast NVMe path; add a cache “packer” that merges per-session shards.

Queue submit strategy: heuristic batching for tiny submits (title-gated).

Metrics hooks: lightweight timestamping of vkQueueSubmit/vkQueuePresentKHR (compiled out in release if env not set).

nvshim (optional, but useful)

A minimal LD_PRELOAD layer that:

Inserts VK_EXT_debug_utils markers around submits/compiles.

Writes a prewarm manifest (list of PSOs/pipelines touched during “warmup run”).

(Experimental) Combine adjacent tiny submits when identical fences/semaphores (strict safety checks).

Profiles (per-game YAML)

Example profiles/diablo4.yaml:

id: blizzard.diablo4
notes: "Reduce PSO churn; prefer present_id+present_wait; VRR on."
env:
  __GL_SHADER_DISK_CACHE: "1"
  __GL_SHADER_DISK_CACHE_PATH: "/mnt/nvme0/ghostcache/blizzard.diablo4"
  DXVK_STATE_CACHE: "1"
  DXVK_STATE_CACHE_PATH: "/mnt/nvme0/ghostcache/blizzard.diablo4"
  VKD3D_NV_PIPELINE_CACHE: "/mnt/nvme0/ghostcache/blizzard.diablo4"
  VKD3D_NV_HEAP_BLOCK: "1M"
  VKD3D_NV_BARRIER_COALESCE: "1"
  __GL_SYNC_TO_VBLANK: "0"
  __GL_GSYNC_ALLOWED: "1"
  __GL_VRR_ALLOWED: "1"
  __GLX_VENDOR_LIBRARY_NAME: "nvidia"
gamescope:
  enable: true
  hz: 165
  vrr: true
  hdr: false
scheduler:
  pin_ccd: 0
  priority: high


Your nvctl launcher can read these and export envs accordingly before launching Steam/Proton.

Proton integration & distribution

Build nvproton as a standard “compatibility tool,” selectable in Steam per title.

Wrapper script:

Applies profile → sets env → calls Proton’s proton run.

If gamescope is enabled in profile, runs through gamescope with the desired flags.

Keep sync with Proton-GE regularly, rebase patches, and publish tags (nvproton-24.9, etc.).

Tooling: cachepack & prewarm

cachepack merges DXVK + vkd3d cache shards into a single file per title, deduping entries.

prewarm runs the game into menu at low res for ~45–60s to populate caches, then exits. Generated manifest improves first real run.

CI & packaging

Build matrix: Arch (Cachy/BORE), Ubuntu LTS, Fedora, Nix.

Produce:

Steam-drop-in tarball (compatibilitytools.d/nvproton/).

OCI image (for nvbind use) with matching userspace libs.

Check:

Vulkan ICD consistency, nvidia-smi --query version match.

Extension availability snapshot (baked into artifacts).

Performance & correctness guardrails

Title-gated risky optimizations (barrier coalesce, submit batching).

Opt-in telemetry (nvmon):

No PII, local JSON logs of frametime, submit→present deltas, shader spikes.

Golden tests:

Replay PSO/manifests, ensure identical render hash before & after optimization.

Wayland / Gamescope integration

Require explicit sync path if available.

Prefer present_id + present_wait if VK_KHR_present_wait is exposed.

Allow VK_EXT_global_priority → HIGH for the graphics queue (configurable).

Milestones

M1 (2–3 weeks)

Forks created; build pipeline; env schema; profiles read by nvctl.

Basic cache relocation; global priority + present_wait when exposed.

Gamescope wrapper + VRR defaults.

M2 (3–5 weeks)

vkd3d-proton-nv: heap allocator modes; PSO hash tweaks; external pipeline cache path.

dxvk-nv: state-cache packer + metrics hook (off by default).

cachepack tool + prewarm script.

M3 (ongoing)

nvshim timing markers; optional batching under strict guards.

Per-title presets; community reports; AB tests vs Proton-GE.

Fast start (today)

Create the wrapper (drop into compatibilitytools.d/nvproton/):

#!/usr/bin/env bash
# nvproton wrapper
set -euo pipefail

GAMEID="${STEAM_COMPAT_DATA_PATH##*/}"
PROFILE_DIR="${HOME}/.config/nvproton/profiles"
PROFILE="${PROFILE_DIR}/${GAMEID}.env"

# Load global + per-title env
[ -f "${PROFILE_DIR}/global.env" ] && export $(grep -v '^#' "${PROFILE_DIR}/global.env" | xargs)
[ -f "${PROFILE}" ] && export $(grep -v '^#' "${PROFILE}" | xargs)

# NVIDIA defaults if not set
export __GL_SHADER_DISK_CACHE="${__GL_SHADER_DISK_CACHE:-1}"
export __GLX_VENDOR_LIBRARY_NAME="${__GLX_VENDOR_LIBRARY_NAME:-nvidia}"

# Prefer fast cache root
export DXVK_STATE_CACHE_PATH="${DXVK_STATE_CACHE_PATH:-/mnt/nvme0/ghostcache/${GAMEID}}"
export VKD3D_NV_PIPELINE_CACHE="${VKD3D_NV_PIPELINE_CACHE:-/mnt/nvme0/ghostcache/${GAMEID}}"
mkdir -p "$DXVK_STATE_CACHE_PATH"

exec "/path/to/proton" run "$@"


Ship a few profiles (PoE2, Diablo IV, WoW, Apex) tuned for your 7950X3D.

Tie into your tools:

nvctl: picks nvproton, applies profile, pins CCD/IRQs, can run prewarm.

nvbind: containerize exact userspace libs for reproducible perf.

Reality check

You can’t replace NVIDIA’s closed userspace or GSP with Zig/Rust. Focus on the translation & timing layers and the system around them.

Most of the “~20%” comes from CPU-side overhead, cache misses, and pacing. The above plan systematically attacks those without breaking legality or stability.
