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
        packages.crunkurrent = naersk-lib.buildPackage {
          pname = "crunkurrent";
          root = ./.;
        };
        defaultPackage = packages.crunkurrent;

        # `nix run`
        apps.crunkurrent = utils.lib.mkApp {
          drv = packages.crunkurrent;
        };
        # apps.cr = apps.crunkurrent;
        defaultApp = apps.crunkurrent;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
        };
      });
}
