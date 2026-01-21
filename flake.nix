{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, systems, fenix, ... }: let
    forEachSystem = nixpkgs.lib.genAttrs (import systems);
  in {
    devShells = forEachSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Runtime libraries for GUI / windowing
        runtimeLibs = if system == "aarch64-darwin" then [] else with pkgs; [
          vulkan-loader
          mesa
          libGL
          fontconfig
          freetype
          expat
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          wayland
          wayland-protocols
          libxkbcommon
        ];

        rustToolchain = fenix.packages.${system}.combine [
          fenix.packages.${system}.stable.toolchain
          fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
        ];
      in {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-outdated
            pkg-config

            wasm-pack
            trunk
          ] ++ runtimeLibs;

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH"
          '';
        };
      });
  };
}
