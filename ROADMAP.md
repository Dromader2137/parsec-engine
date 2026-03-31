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
