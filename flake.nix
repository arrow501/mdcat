{
  description = "mdcat with sixel image support and runtime terminal capability detection";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "mdcat";
          version = "2.7.1";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
            installShellFiles
          ];

          buildInputs = with pkgs; [
            curl
            openssl
          ];

          postInstall = ''
            installShellCompletion --cmd mdcat \
              --bash <($out/bin/mdcat --completions bash) \
              --fish <($out/bin/mdcat --completions fish) \
              --zsh  <($out/bin/mdcat --completions zsh)
          '';

          meta = with pkgs.lib; {
            description = "cat for markdown with sixel image support";
            homepage = "https://github.com/arrow501/mdcat";
            license = licenses.mpl20;
            mainProgram = "mdcat";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            curl
            openssl
          ];
        };
      }
    );
}
