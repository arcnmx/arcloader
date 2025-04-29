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
  stripAll = buildType == "release";
in rustPlatform.buildRustPackage {
  pname = crate.name;
  inherit (crate) version;

  src = source;
  inherit cargoLock buildType;
  doCheck = false;

  ${if stripAll then "stripAllList" else null} = ["bin"];

  postInstall = ''
    rm -f $out/lib/lib${crate.name}${hostPlatform.extensions.sharedLibrary}.a
    rmdir --ignore-fail-on-non-empty $out/lib
  '';

  #"CFLAGS_x86_64_pc_windows_gnu" = cxxflags;
  #"CXXFLAGS_x86_64_pc_windows_gnu" = cxxflags;

  meta = {
    license = licenses.mit;
    maintainers = [ maintainers.arcnmx ];
    #platforms = platforms.windows;
  };
}
