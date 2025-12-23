# Parsec Engine

Simple, modular, ergonomic and high-performance game engine tailored for space scale games.

> [!CAUTION]
> Pull requests won't be accepted for the time being!

## Features

- High-performance modular ECS
- Global state queryable resources
- Custom Vulkan wrapper
- Extendable built-in renderer
- Proper input handling

## Design goals

- Invisible spatial partitioning allowing for universe scale games without special systems (floating origin etc)
- Ergonomic design moving the boilerplate away from the user, but also allowing low level access
- Modularity

## TODO

- [x] Timing resource (delta time, uptime, etc)
- [x] Textures
- [ ] Fix Transform and Cameara data artifacts
- [ ] Proper gpu buffer allocator that generates the buffers on the gpu and not cpu !!!!!!!!
- [ ] Better OBJ loader (mesh splitting, auto material generation, etc)
