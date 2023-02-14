{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
      rec {
        # `nix build`
        packages.default = naersk-lib.buildPackage {
          pname = "crunkurrent";
          root = ./.;
        };

        # `nix run`
        apps.crunkurrent = utils.lib.mkApp {
          drv = packages.default;
        };
        apps.cr = apps.crunkurrent;
        apps.default = apps.crunkurrent;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; ([ rustc cargo rustfmt ] ++ lib.optionals stdenv.isDarwin [ libiconv ]);
        };
      });
}
