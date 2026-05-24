{ pkgs, ... }:

{
  packages = [ pkgs.git ];

  languages.rust.enable = true;
  languages.rust.mold.enable = true;
}
