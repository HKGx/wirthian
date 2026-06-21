{ pkgs, ... }:

{
  packages = [ pkgs.git pkgs.binaryen ];

  languages.rust.enable = true;
  languages.rust.channel = "stable";
  languages.rust.targets = [ "wasm32-unknown-unknown" ];

  languages.javascript.enable = true;
  languages.javascript.npm.enable = true;
  languages.javascript.npm.install.enable = true;
  languages.javascript.directory = "web";

  processes.wasm-build.exec = "cargo watch -w src -s 'wasm-pack build --target bundler -- --no-default-features --features wasm'";
  processes.web-dev.exec = "npm --prefix web run dev -- --host";
}
