cp target/release/sysY .
cargo build --manifest-path Cargo.toml --release
# docker run -it --rm -v ./:/root/compiler maxxing/compiler-dev-updated autotest -koopa -s lv1 /root/compiler
docker run -it --rm -v ./:/root/compiler maxxing/compiler-dev-updated autotest -riscv -s lv4 /root/compiler
rm sysY
