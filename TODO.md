# TODO

## Bugfixes
- Vulkan buffer automatic barrier and synchronization (DONE)
- OBJ loader support

## Features
- Asset lazy loading (may require some lifetime shenannigans)
- UI
- Create an easy way to bake in some default shaders into the engine
- Decouple fetch from filters inside the ECS
- CI/CD on pull request to main that uploads to crates.io

## Optimization
- Switch mutex in BorrowingStats to atomic int

## Other
- Document Resources
- Document World
- Document Graphics
- Vulkan tests
- World tests
- Deny unwrap
- Derive error for all error traits
