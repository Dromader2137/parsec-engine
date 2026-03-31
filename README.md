# Parsec Engine

Simple, ergonomic and high-performance game engine made specifically for space scale games.

> [!CAUTION]
> I am not acceping any pull requests for the time being!

## Features

- High performance ECS
- Optimized built-in renderer

## Design goals

- Invisible spatial partitioning allowing developers to create universe scale games without special systems like "floating origin"
- Ergonomic design moving the boilerplate away from the user, but also allowing low level access if you need it
- High performance even on integrated GPUs
- Perfect Wayland support on Linux

## Activity

### Currently in progress

- Importing assets

### Plans for the near future

- Project management
- Better .obj loader (objects, shading, mtl...)

### Already done

- Shadowmaps
- Performant Vulkan buffer allocator
- Timing resource (delta time, uptime, start time...)
- Textures
- Basic renderer
- ECS
