with import <nixpkgs> {}; {
  devEnv = stdenv.mkDerivation {
    name = "dev";
    buildInputs = [ stdenv openssl pkgconfig rustup  ];
  };
}

