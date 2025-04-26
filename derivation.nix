let
  self = import ./. { pkgs = null; system = null; };
in {
  rustPlatform
, hostPlatform
, lib
, buildType ? "release"
, cargoLock ? crate.cargoLock
, source ? crate.src
, crate ? self.lib.crate
}: with lib; let
  cflags = [ "-Oz" ];
in rustPlatform.buildRustPackage {
  pname = crate.name;
  inherit (crate) version;

  src = source;
  inherit cargoLock buildType;
  doCheck = false;

  "CFLAGS_x86_64_pc_windows_gnu" = toString cflags;
  "CXXFLAGS_x86_64_pc_windows_gnu" = toString cflags;

  meta = {
    license = licenses.mit;
    maintainers = [ maintainers.arcnmx ];
    platforms = platforms.windows;
  };
}
