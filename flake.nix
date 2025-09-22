{
  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs = {nixpkgs.follows = "nixpkgs";};
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = {
    self,
    nixpkgs,
    devenv,
    systems,
    ...
  } @ inputs: let
    forEachSystem = nixpkgs.lib.genAttrs (import systems);
  in {
    packages = forEachSystem (system: {
      devenv-up = self.devShells.${system}.default.config.procfileScript;
      devenv-test = self.devShells.${system}.default.config.test;
    });

    devShells =
      forEachSystem
      (system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        default = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            rec {
              languages.rust.enable = true;
              languages.rust.channel = "nightly";
              packages = with pkgs; [

                vulkan-loader
                expat
                fontconfig
                freetype
                freetype.dev
                pkg-config
                xorg.libX11
                xorg.libXcursor
                xorg.libXi
                xorg.libXrandr
                wayland
                libxkbcommon
              ];
              enterShell = ''
                export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath packages)}";
              '';
            }
          ];
        };
      });
  };
}

# {
#   inputs = {
#     nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
#     systems.url = "github:nix-systems/default";
#   };

#   outputs = { nixpkgs, systems, ... }:
#     let
#       eachSystem = nixpkgs.lib.genAttrs (import systems);
#       pkgsFor = nixpkgs.legacyPackages;
#     in {
#       devShells = eachSystem (system:
#         let
#           pkgs = pkgsFor.${system};
#           dlopenLibraries = with pkgs; [
#             # libxkbcommon

#             # # GPU backend
#             # vulkan-loader
#             # # libGL

#             # # Window system
#             # wayland
#             # xorg.libX11
#             # xorg.libXcursor
#             # xorg.libXi

#             vulkan-loader
#             expat
#             fontconfig
#             freetype
#             freetype.dev
#             pkg-config
#             xorg.libX11
#             xorg.libXcursor
#             xorg.libXi
#             xorg.libXrandr
#             wayland
#             libxkbcommon
#           ];
#         in {
#           default = pkgs.mkShell {
#             nativeBuildInputs = with pkgs; [
#               cargo
#               rustc
#             ];

#             env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${nixpkgs.lib.makeLibraryPath dlopenLibraries}";
#           };
#         });
#     };
# }
