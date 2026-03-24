# Parsec Engine Roadmap

## Phase 1: Vulkan Abstraction Fixes (DONE, FOR REFACTOR)

### 1.1 GPU Memory Suballocator (DONE)

### 1.2 Resource Cleanup / Drop (DONE)

### 1.3 Staging Buffer Uploads (FOR REFACTOR)

## Phase 2: Vulkan Abstraction Improvements (DONE, FOR LATER)

### 2.1 Replace Backtracking Command Buffer (DONE)

### 2.2 Configurable Vertex Formats (DONE)

### 2.3 Descriptor Set Image Tracking (DONE)

### 2.4 Lock-Free ID Counters (DONE)

### 2.5 Multi-Queue Support (FOR LATER)
- Add support for dedicated transfer and compute queues
- Enable async texture uploads on the transfer queue while rendering continues
- Enable parallel compute work on a separate compute queue

## Phase 3: Multi-Backend Abstraction

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
