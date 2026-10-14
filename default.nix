{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  pname = "cpu-z-bench-linux";
  version = "unstable";

  src = pkgs.lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let
        name = builtins.baseNameOf (toString path);
      in
      pkgs.lib.cleanSourceFilter path type
      && name != "cpu-z-bench-linux"
      && name != "result"
      && !(pkgs.lib.hasPrefix "result-" name);
  };
  nativeBuildInputs = [ pkgs.gnumake ];

  dontConfigure = true;

  buildPhase = ''
    runHook preBuild
    make
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 cpu-z-bench-linux "$out/bin/cpu-z-bench-linux"
    install -Dm644 README.md "$out/share/doc/cpu-z-bench-linux/README.md"
    install -Dm644 LICENSE "$out/share/licenses/cpu-z-bench-linux/LICENSE"
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "Independent CPU-Z-like CPU benchmark for Linux";
    homepage = "https://github.com/kimlherme/cpuz-linux";
    license = licenses.gpl2Only;
    mainProgram = "cpu-z-bench-linux";
    platforms = platforms.linux;
  };
}
