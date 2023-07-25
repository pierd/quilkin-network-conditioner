fn main() {
    // Remove if you already have `protoc` installed in your system.
    std::env::set_var("PROTOC", protobuf_src::protoc());

    prost_build::compile_protos(&["src/delay.proto"], &["src/"]).unwrap();
}
