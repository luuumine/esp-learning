{
  description = "ESP32 Xtensa Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    esp-rs-nix = {
      url = "github:leighleighleigh/esp-rs-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      esp-rs-nix,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      esp-rust = esp-rs-nix.packages.${system}.esp-rs;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          esp-rust
          rust-analyzer
          espflash
          cargo-generate
        ];

        shellHook = ''
          export RUSTUP_TOOLCHAIN=${esp-rust}
          echo "ESP32 Xtensa development environment active"
        '';
      };
    };
}
