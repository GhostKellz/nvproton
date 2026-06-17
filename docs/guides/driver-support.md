# Driver Support

nvproton's DX12 optimizations depend on a set of Vulkan extensions that NVIDIA
first shipped in the **595** driver branch and carries forward in every later
branch (600, 610, ...). Detection is driven by **extension probing**, not by a
specific version number, so newer drivers keep working without code changes.

## How detection works

`nvproton status` loads Vulkan, finds the NVIDIA GPU, and records:

- the reported driver version and branch (e.g. `610.43.02`, branch `610`)
- which DX12 / latency extensions the driver exposes

The driver branch is only used as a **floor** for messaging and auto-enable
fallbacks. If the relevant extension is present, it is used regardless of branch.

```bash
nvproton status            # human-readable report + recommendations
nvproton status --verbose  # also prints the driver branch
nvproton status --check    # exit 0 if descriptor_heap is ready, else 1
```

## Feature tiers

| Level | Requirement | What you get |
|-------|-------------|--------------|
| 0 | No NVIDIA GPU, or branch &lt; 580 | No DX12 heap optimizations |
| 1 | `VK_EXT_descriptor_heap` (branch 580.94+) | DX12 descriptor mapping |
| 2 | + `VK_NV_extended_sparse_address_space` (595+) | DX12 heap fix |
| 3 | + `VK_NV_low_latency2` (595+) | Full set incl. Reflex 2.0 |

`is_fully_supported()` (level 3) requires `descriptor_heap`,
`extended_sparse_address_space`, and `low_latency2` together.

## Extensions tracked

| Extension | Purpose |
|-----------|---------|
| `VK_EXT_descriptor_heap` | Primary DX12 descriptor mapping fix |
| `VK_EXT_descriptor_buffer` | Fallback descriptor path |
| `VK_NV_extended_sparse_address_space` | DX12 heap fix |
| `VK_NV_raw_access_chains` | Shader access optimization |
| `VK_NV_low_latency2` | Reflex 2.0 |
| `VK_EXT_present_timing` | Frame pacing |

## Branch classification

- **DX12-capable floor:** branch `>= 595` (`DX12_HEAP_FIX_MIN_BRANCH`). All
  595, 600, and 610 branches clear this floor.
- **Beta branches:** `580.x` and `595.x` are treated as beta/Vulkan-dev series.
  Production branches such as `590`, `600`, and `610` are not flagged as beta.

These ranges live in `BETA_BRANCH_RANGES` in `src/detection/vulkan.rs`; add a
range there if NVIDIA ships a new beta series.

## vkd3d-proton

The driver extensions are only half of the picture — vkd3d-proton must also
consume `descriptor_heap`. `nvproton status` reports the detected vkd3d-proton
version and whether it advertises descriptor_heap support, and `nvproton run`
sets `VKD3D_CONFIG` / `VKD3D_FEATURE_LEVEL` accordingly. Auto-enable behavior is
controlled by `vkd3d.auto_enable_dx12_heap` in the config (see
[Configuration](../getting-started/configuration.md)).
