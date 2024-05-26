# vterm

### a cross platform, vulkan terminal emulator written in rust.

# deps:

- vulkan sdk
- rust nightly
- just (optional)

# usage:

run it using just:

```bash
just build
just run
```

or just copy out the build script out of the [justfile](./justfile) and run it manually.

#### note: is your platform not supported? either wait or contribute.

#### note: there is a small execution barrier (200ms~) before the terminal opens on NVIDIA cards, this is a driver related issue. this has something to do with vkCreateInstance and vkCreateDevice being extremely slow on NVIDIA cards. I am hoping to improve it as much as possible, but the biggest overhead lies in the lack of driver optimization. So I think we can expect improvements soon.
