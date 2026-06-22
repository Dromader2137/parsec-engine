# TODO

## Bugfixes
- Vulkan buffer barrier system (DONE)

## Features
- Asset lazy loading
- Small buffer Host allocation that synchronizes well
    Now everything uses a staging buffer which may not be ideal

## Optimization
- BorrowingStats atomics instead of mutexes
- Batch buffer and image data transfers into a single submit between frames

## Other
- Resources docs
- World docs
- Graphics docs
