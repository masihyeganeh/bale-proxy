fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile(&["proto/auth.proto"], &["proto"])
        .unwrap();
}
