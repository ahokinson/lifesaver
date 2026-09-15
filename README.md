# LifeSaver

LifeSaver is a fullscreen, Catppuccin Mocha Conway's Game of Life display for
Linux and macOS. It is deliberately **not** an idle daemon: the desktop binds a
key to launch it, and it never starts on its own.

By default it opens fullscreen and closes on any key press, click, scroll, or
meaningful pointer movement — the expected screensaver behaviour. The wrapped
board uses Conway's B3/S23 rules. It re-seeds immediately on extinction, when
the last 90 generations average fewer than 0.4% changed cells after generation
270, or after 1,800 generations (about three minutes at the default speed).

The board is sized from the active viewport at roughly 12 pixels per cell,
bounded to keep very high-resolution displays responsive; a material resize
starts a matching new seed. Fullscreen mode hides the pointer and restores it
when the screensaver exits.

Isolated structures use a semantic Catppuccin accent: any connected structure
that remains unchanged for a generation is a still life and uses Overlay2;
oscillators are green, gliders blue, and lightweight spaceships mauve.
Every other connected live cluster shares a smoothly cycling accent drawn from
the unused Mocha palette colours, so no shape is split across unrelated colours.
There is no visual legend, grid, or HUD on-screen; the README is deliberately
the only place the colour language is spelled out.

## Run

```sh
nix run .#
```

For a windowed, controllable preview:

```sh
nix develop -c cargo run --release -- --windowed --interactive
```

The development shell supplies Linux's dynamically loaded X11 compatibility
libraries, which plain `cargo run` cannot discover on NixOS. On macOS, normal
`cargo run --release` works as expected.

Interactive controls are `Space` (pause), `R` (reseed), `Up`/`Down` (speed),
and `Esc` or `Q` (quit). There is deliberately no on-screen HUD or grid: every
lit pixel is part of the evolving simulation. The default screensaver mode
exits on input instead.

Optional tuning flags work in either mode:

```sh
lifesaver --density 0.18 --speed 12
```

## Nix

The flake exports `packages.<system>.default` and an overlay, with Linux and
Apple Silicon macOS support:

```sh
nix run github:ahokinson/lifesaver
```

The companion dotfiles configuration supplies explicit shortcut bindings; it
does not attach LifeSaver to any idle or lock event.
