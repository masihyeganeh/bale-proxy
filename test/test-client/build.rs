fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &[
                "../../bale/proto/auth.proto",
                "../../bale/proto/messaging.proto",
                "../../bale/proto/maviz.proto",
            ],
            &["../../bale/proto"],
        )
        .unwrap();
}
