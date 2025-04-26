{ inputs
, pkgs
, pkgs-w64
, rust, rust-w64-overlay
}:
let
  inherit (pkgs) system;
  fenix' = inputs.fenix.packages.${system};
  fenix'shell = { mkShell, lib, buildPackages, stdenv, windows }: mkShell rec {
    buildInputs = [
      stdenv.cc
      windows.pthreads
    ];

    nativeBuildInputs = [
      buildPackages.stdenv.cc
      (fenix'.combine [
        (fenix'.complete.withComponents [
          "cargo"
          "rust-src"
          #"clippy"
          "rustc"
        ])
        fenix'.rust-analyzer
        #fenix'.latest.rustfmt
        fenix'.targets.x86_64-pc-windows-gnu.latest.rust-std
      ])
    ];

    CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
    TARGET_CC = "${stdenv.cc.targetPrefix}cc";
    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = TARGET_CC;
    CXXFLAGS_x86_64_pc_windows_gnu = "-shared -fno-threadsafe-statics";
  };
  arc'shell = { lib, buildPackages, stdenv, windows }: let
    channel = rust.unstable.override {
      channelOverlays = [ rust-w64-overlay ];
    };
  in channel.mkShell {
    rustTools = ["rust-analyzer"];
    nativeBuildInputs = [
      pkgs-w64.stdenv.cc.bintools
      #pkgs-w64.stdenv.cc
      #pkgs-w64.clangStdenv.cc
    ];

    CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
  };
in {
  default = inputs.self.devShells.${system}.arc;
  fenix = pkgs-w64.callPackage fenix'shell { };
  arc = pkgs.callPackage arc'shell { };
}
