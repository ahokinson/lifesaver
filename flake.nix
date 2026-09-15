{
  description = "LifeSaver — a Catppuccin Mocha Conway's Game of Life screensaver";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs { inherit system; };
          in
          f pkgs
        );
    in
    {
      packages = forAllSystems (pkgs:
        let
          package = pkgs.callPackage ./package.nix { };
        in
        {
          default = package;
          lifesaver = package;
        }
      );

      # `cargo run` needs the same dynamically-loaded X11 libraries as the
      # packaged program. Use `nix develop -c cargo run --release` while
      # iterating on Linux; macOS needs no equivalent library-path setup.
      devShells = forAllSystems (pkgs:
        let
          runtimeLibraries =
            if pkgs.stdenv.hostPlatform.isLinux then
              with pkgs;
              [
                libGL
                libxkbcommon
                libx11
                libxcursor
                libxi
                libxrandr
              ]
            else
              [ ];
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              pkg-config
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibraries;
          };
        }
      );

      overlays.default = final: _: {
        lifesaver = self.packages.${final.stdenv.hostPlatform.system}.default;
      };
    };
}
