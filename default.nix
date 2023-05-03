# default.nix
{ lib
, naersk
, stdenv
, clangStdenv
, hostPlatform
, targetPlatform
, pkg-config
, libiconv
, rustfmt
, cargo
, rustc
  , llvmPackages
}:

let
  cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
in

naersk.lib."${targetPlatform.system}".buildPackage rec {
  src = ./.;

  buildInputs = [
    rustfmt
    pkg-config
    cargo
    rustc
    libiconv
  ];
  checkInputs = [ cargo rustc ];

  doCheck = true;
  CARGO_BUILD_INCREMENTAL = "false";
  RUST_BACKTRACE = "full";
  copyLibs = true;

  LIBCLANG_PATH = "${llvmPackages.libclang}/lib";

  name = cargoToml.package.name;
  version = cargoToml.package.version;

  meta = with lib; {
    description = cargoToml.package.description;
    homepage = cargoToml.package.homepage;
    license = with licenses; [ mit ];
    maintainers = with maintainers; [ ];
  };
}

