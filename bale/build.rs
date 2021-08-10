fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &[
                "proto/configs.proto",
                "proto/auth.proto",
                "proto/messaging.proto",
                "proto/maviz.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
