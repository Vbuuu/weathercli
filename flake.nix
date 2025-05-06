{
  description = "A Simple Weather Cli";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      systems,
      rust-overlay,
    }@inputs:
    let
      forAllSystems = nixpkgs.lib.genAttrs (import systems);
      pkgsFor = nixpkgs.legacyPackages;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              (import rust-overlay)
            ];
          };

          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
            rustc = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          };
        in
        {
          default =
            let
              manifest = pkgs.lib.importTOML ./Cargo.toml;
            in
            rustPlatform.buildRustPackage {
              pname = manifest.package.name;
              version = manifest.package.version;

              src = pkgs.lib.cleanSource ./.;

              cargoLock.lockFile = ./Cargo.lock;
              doCheck = false;

              nativeBuildInputs = with pkgs; [ pkg-config ];
              buildInputs = with pkgs; [ openssl ];

              meta = with pkgs.lib; {
                description = manifest.package.description;
              };
            };
        }
      );

      homeManagerModules = {
        weather-cli = import ./module.nix self;
        default = self.homeManagerModules.weather-cli;
      };
    };
}
