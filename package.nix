{
  lib,
  stdenv,
  rustPlatform,
  pkg-config,
  makeWrapper,
  libGL,
  libxkbcommon,
  libx11,
  libxcursor,
  libxi,
  libxrandr,
}:
let
  runtimeLibraries = lib.optionals stdenv.hostPlatform.isLinux [
    libGL
    libxkbcommon
    libx11
    libxcursor
    libxi
    libxrandr
  ];
in
rustPlatform.buildRustPackage {
  pname = "lifesaver";
  version = "0.1.0";

  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  buildInputs = runtimeLibraries;

  # miniquad opens these libraries itself rather than linking to them, so an
  # ELF RPATH alone is not enough. The wrapper is also what makes the package
  # work inside a Wayland session through XWayland.
  postFixup = lib.optionalString stdenv.hostPlatform.isLinux ''
    wrapProgram "$out/bin/lifesaver" \
      --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath runtimeLibraries}"
  '';

  meta = {
    description = "A Catppuccin Mocha fullscreen Conway's Game of Life screensaver";
    homepage = "https://github.com/ahokinson/lifesaver";
    license = lib.licenses.mit;
    mainProgram = "lifesaver";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
}
