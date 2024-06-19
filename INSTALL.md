# Installation

We do not currently have a binary available for installation.
However, you can build it yourself from source or install it via the AUR if you are on Arch Linux.

## Platforms

- [Arch Linux](#arch-linux)
- [Manual from Source](#manual-from-source)
- [Windows (Work in Progress)](#windows)
- [macOS (Work in Progress)](#macos)

## Arch Linux

You can install `vterm` via the AUR:

```sh
yay -S vterm-git
```

### Dependencies

To build `vterm` on Arch Linux, you need the following dependencies:

- `vulkan-icd-loader` (or `vulkan-icd-loader-git` from AUR)
- `cargo` (or `rust-nightly-bin`, `cargo-git`, `rustup-git`, `rust`, `rustup` from AUR) (make)
- `git` (or `git-git` from AUR) (make)
- `just` (or `just-git` from AUR) (make)
- `rustup` (or `rustup-git` from AUR) (make)
- `vulkan-headers` (or `vulkan-headers-git` from AUR) (make)

## Manual from Source

### Dependencies

To build `vterm` from source, you need the following dependencies:

- `vulkan-icd-loader`
- `cargo`
- `git`
- `just`
- `rustup`
- `vulkan-headers`

### Building

To run the binary directly, use:

```sh
cargo run
cargo run --release
```

### Optimized Build

For a highly optimized build, use `just`:

```sh
just build
just run
```

Or manually run the commands from the `justfile`:

```sh
cargo build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu --release
cargo run -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu --release -- --disable-validation
```

## Windows (Work in Progress)

Support for Windows is currently a work in progress. Please check back later for updates.

## macOS (Work in Progress)

Support for macOS is currently a work in progress. Please check back later for updates.
