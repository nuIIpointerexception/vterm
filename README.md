# vterm

### A cross-platform, Vulkan terminal emulator written in Rust.

### The project is not usable in its current state and is subject to change a lot.

### Project Status
- [x] Windowing
- [x] Rendering pipeline
- [x] Image drawing
- [x] Font drawing
- [x] Custom font loading
- [ ] Layouting for UI layer (Bugged right now)
- [ ] Ptty
- [ ] Handle input
- [ ] Terminal buffer to screen
- [ ] Tabs
- [ ] Draggable / multi window
- [ ] Menus, settings
- [ ] Config

## Dependencies

- Vulkan SDK
- Rust nightly

## Installation

For detailed installation instructions, please refer to the [INSTALL.md](./INSTALL.md) file.

#### Note: Is your platform not supported? Either wait or contribute.

#### Note: There is a small execution barrier (~200ms) before the terminal opens on NVIDIA cards. This is a driver-related issue. It has something to do with `vkCreateInstance` and `vkCreateDevice` being extremely slow on NVIDIA cards. I am hoping to improve it as much as possible, but the biggest overhead lies in the lack of driver optimization. So I think we can expect improvements soon.