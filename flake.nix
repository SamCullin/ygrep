{
  description = "Development environment for ygrep";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        workspaceConfig = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        version = workspaceConfig.workspace.package.version or "dev";

        ygrep = pkgs.rustPlatform.buildRustPackage {
          pname = "ygrep";
          inherit version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [
            "--package" "ygrep-cli"
            "--bin" "ygrep"
            "--all-features"
          ];
          cargoInstallFlags = [
            "--path" "crates/ygrep-cli"
            "--all-features"
          ];
          doCheck = false;
          strictDeps = true;
          meta = with pkgs.lib; {
            description = "Fast indexed code search CLI optimized for AI assistants";
            homepage = "https://github.com/yetidevworks/ygrep";
            license = licenses.mit;
            mainProgram = "ygrep";
            platforms = platforms.unix;
          };
        };

        devPackages = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          pkg-config
          cmake
          openssl
          python3
        ];
      in
      {
        packages = {
          default = ygrep;
          inherit ygrep;
        };

        apps = {
          default = flake-utils.lib.mkApp { drv = ygrep; };
          ygrep = flake-utils.lib.mkApp { drv = ygrep; };
        };

        devShells.default = pkgs.mkShell {
          packages = devPackages;
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
