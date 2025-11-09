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
        runtimeLibs = with pkgs; [
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

        rust = fenix.packages.${system}.stable;
      in {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rust.toolchain
            cargo-outdated
            pkg-config
          ] ++ runtimeLibs;

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH"
          '';
        };
      });
  };
}
