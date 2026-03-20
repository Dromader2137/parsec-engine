# Parsec Engine Roadmap

## Phase 1: Vulkan Abstraction Fixes (DONE, parts may need a refactor, but not urgently)

These are blocking issues that will prevent the abstraction from scaling to real workloads.

### 1.1 GPU Memory Suballocator (DONE)
- Implement a suballocator in `allocator.rs`/`allocation.rs`
- Replace per-resource `vkAllocateMemory` calls in `buffer.rs` and `image.rs` with suballocations
- Vulkan drivers guarantee only ~4096 simultaneous allocations; a non-trivial scene will hit this limit

### 1.2 Resource Cleanup / Drop (DONE)
- Implement `Drop` for `VulkanBackend` that destroys all resources in its HashMaps
- Implement `Drop` for individual wrapper types (or ensure the backend always cleans up)
- Fix `delete_swapchain` to actually call `vkDestroySwapchainKHR`
- Audit all `delete_*` methods for missing Vulkan destroy calls

### 1.3 Staging Buffer Uploads (DONE, needs a refactor down the line)
- Replace `HOST_VISIBLE | DEVICE_LOCAL` allocation for vertex/index/large uniform buffers
- Implement staging buffer pattern: write to `HOST_VISIBLE` staging, `vkCmdCopyBuffer` to `DEVICE_LOCAL`
- Keep `HOST_VISIBLE` only for small, frequently-updated uniform buffers
- This is required for discrete GPU compatibility (HOST_VISIBLE + DEVICE_LOCAL is 256MB or 0 without ReBAR)

### 1.4 Synchronization Fixes (DONE)
- Fix `load_image_from_buffer`: submit with a real fence and wait, or call `vkQueueWaitIdle`
- Make `wait_dst_stage_mask` dynamic based on actual submission content instead of hardcoded `COLOR_ATTACHMENT_OUTPUT`

## Phase 2: Vulkan Abstraction Improvements

Architectural improvements that reduce tech debt and unlock future features.

### 2.1 Replace Backtracking Command Buffer (DONE)
- Remove the record-then-replay pattern (`*_backtrack` methods) in `command_buffer.rs`
- Options:
  - Pre-compute barriers before the renderpass starts (descriptor sets are known at that point) **THIS**
  - Use render pass self-dependencies for mid-pass barriers
  - Adopt `VK_KHR_synchronization2` for more granular barrier control
- This eliminates the duplicated API surface and makes RenderDoc debugging intuitive

### 2.2 Flatten Command Indirection (REJECTED)
- Evaluate whether the backend-agnostic `Command` enum layer is needed
- Currently commands pass through three layers: `Command` enum -> `submit_commands` lookup -> `VulkanCommand` buffer -> replay
- Consider recording directly through the builder if the `Command` enum doesn't add real portability

### 2.3 Configurable Vertex Formats
- Remove hardcoded `DefaultVertex` in pipeline creation
- Pass vertex attribute descriptions from the caller
- Required for skinned meshes, particles, debug lines, or any non-standard vertex layout

### 2.4 Descriptor Set Image Tracking
- Fix append-only `bound_image_ids` in descriptor sets
- Clear or replace tracked image IDs on rebind to avoid stale barrier insertions in `end_renderpass`

### 2.5 Lock-Free ID Counters (DONE)
- Replace `Mutex<u32>` in `IdCounter` with `AtomicU32::fetch_add`
- Minor, but removes unnecessary contention if resources are ever created from multiple threads

### 2.6 Multi-Queue Support
- Add support for dedicated transfer and compute queues
- Enable async texture uploads on the transfer queue while rendering continues
- Enable parallel compute work on a separate compute queue

## Phase 3: Multi-Backend Abstraction

Redesign the `GraphicsBackend` trait to support non-Vulkan backends.

### 3.1 Raise the Abstraction Level
- Current trait exposes Vulkan-specific concepts (semaphores, fences, explicit command begin/end, swapchain image acquisition)
- Metal has no user-visible semaphores; D3D12 uses a different sync model
- Redesign the trait around frames and passes rather than sync primitives:
  - `begin_frame()` / `end_frame()` instead of manual swapchain acquire + present + fence management
  - Render pass descriptions instead of explicit renderpass + framebuffer pairs
  - Resource binding groups instead of raw descriptor sets

### 3.2 Backend-Internal Synchronization
- Move fence/semaphore management inside the backend implementation
- The frontend should express data dependencies (e.g., "this pass reads texture X written by pass Y")
- Each backend translates dependencies to its native sync model

### 3.3 Backend-Owned Resource Handles
- Replace `u32` IDs with typed, opaque handles (e.g., `BufferHandle`, `ImageHandle`)
- Prevents accidental cross-type lookups (buffer ID 5 vs image ID 5 in the wrong HashMap)
- Each backend can store whatever internal data it needs behind the handle

### 3.4 Shader Abstraction
- Define a shader IR or interchange format (SPIR-V as source of truth, transpile for other backends)
- Or use a shading language that compiles to multiple targets (e.g., Naga/WGSL, SPIRV-Cross)

### 3.5 Platform Backends
- D3D12 backend (Windows)
- Metal backend (macOS/iOS)
- Each backend implements the redesigned high-level trait from 3.1
