# Muffin

A simple TUI for managing `tmux` sessions.

_This project is heavily inspired by
[`muxie`](https://github.com/phanorcoll/muxie), borrowing many functional and
design decisions from it. Go check it out and show your support!_

## Usage

```
Usage: muffin [OPTIONS]

OPTIONS:
    -s, --start-preset <NAME>   Start preset
    -l, --list-presets          List presets information
    -p, --presets <FILE>        Path to presets file [default: ~/.config/muffin/presets.kdl]
    -e, --exit-on-switch        Close muffin after switching to a session/preset
    -h, --help                  Print help
```

While `muffin` can be run from the command line, it's power is best utilized
when bound to a key within `tmux`.

For example, my `tmux.conf` includes the following:

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



https://github.com/user-attachments/assets/d536f827-b70e-440f-9679-97097357aaa1



## Building

The release profile is currently designed to optimize for a minimal binary size. 
My reasons for this are simple:

1. The actual TUI application is snappy enough and bound via I/O blocking, so
   speed optimizations are really a fool's errand.
2. It's funny.

### Cargo

To build muffin, simply run:

```
cargo build --release
```

and you should be good to go!

### Nix

A simple `flake.nix` is also provided with `muffin` exposed as a package. This means
you can run with `nix run github:zSuperx/muffin`.

To properly add `muffin` to your `$PATH`, first add it to your flake inputs:
```nix
{
  inputs.muffin.url = "github:zSuperx/muffin";
  # ...
}
```
then add the following to your `configuration.nix` or adjacent:
```nix
{ inputs, ... }:
let
  system = "x86_64-linux"; # or your system
in
{
  environment.systemPkgs = [
    inputs.muffin.packages.${system}.muffin
  ];
}
```
