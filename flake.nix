{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = nixpkgs.lib.genAttrs systems;
    in {
      packages = forAll (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "drift-plus";
            version = "1.3";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.makeWrapper ];
            postInstall = ''
              wrapProgram $out/bin/drift-plus \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.mpv pkgs.yt-dlp ]}
            '';
            meta.mainProgram = "drift-plus";
          };
        });
    };
}
