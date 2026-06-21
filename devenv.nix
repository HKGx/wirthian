{ pkgs, ... }:

{
  packages = [ pkgs.git pkgs.binaryen ];

  languages.rust.enable = true;
  languages.rust.channel = "stable";
  languages.rust.targets = [ "wasm32-unknown-unknown" ];
}
