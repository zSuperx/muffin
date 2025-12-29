# Muffin

A simple TUI for managing `tmux` sessions.

_This project is heavily inspired by
[`muxie`](https://github.com/phanorcoll/muxie), borrowing many functional and
design decisions from it. Go check it out and show your support!_

## Usage

```
Usage: muffin [OPTIONS]

OPTIONS:
    -p, --presets <FILE>    Path to KDL file with session presets
    -h, --help              Print help
```

While `muffin` can be run from the command line, it's power is best utilized
when bound to a key within `tmux`.

For example, my config includes the following:

```tmux
# Override tmux's builtin session manager with muffin
unbind s
bind s popup -EB /path/to/muffin
bind -n M-s popup -EB /path/to/muffin # `Alt + s` as a nice shortcut
```

_(Hint: if you generate your tmux config file with `Nix`, you can replace
`/path/to/muffin` with `${lib.getExe muffin}`, where `muffin` points to this
flake's package derivation)_


The following demo runs with [`presets.kdl`](examples/presets.kdl):

<video src="https://github.com/zSuperx/muffin/raw/refs/heads/master/examples/muffin-demo.mp4" controls="controls" style="max-width: 100%;">
</video>

## Building

The release profile is currently designed to optimize for a minimal binary size. 
My reasons for this are simple:

1. The actual TUI application is snappy enough and bound via I/O blocking, so
   speed optimizations are really a fool's errand.
2. It's funny.

### Cargo

This project _may_ require a nightly compiler to build the optimized release
profile, so if that's something you don't want to install, simply comment out
the entire `[profile.release]` block from the `Cargo.toml`. In either case,
simply run:

```
cargo build --release
```

and you should be good to go!

_(I'm saying "may" because it's 3am at the time of writing this and I cba to
actually find out)_

### Nix

A simple `flake.nix` is also provided with `muffin` exposed as a package. This means
you can run with `nix run github:zSuperx/muffin`.
