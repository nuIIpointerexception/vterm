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

#### note: there is a execution delay (200ms~) before the terminal opens on nvidia cards, this is a driver related issue and will surely improve in the future.
