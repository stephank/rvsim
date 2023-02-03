with import <nixpkgs> { };
mkShell {
  nativeBuildInputs = [ pkgsCross.riscv32-embedded.buildPackages.gcc-unwrapped ]
    ++ (with rust; [ cargo clippy rustfmt ])
    ++ lib.optional stdenv.isDarwin [ libiconv ];
}
