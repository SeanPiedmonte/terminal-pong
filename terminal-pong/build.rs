
fn main() {
    protobuf_codegen::Codegen::new()
        // Pure-Rust .proto parser: no system `protoc` install needed.
        .pure()
        .cargo_out_dir("proto")
        .include("proto")
        .input("proto/message.proto")
        .run_from_script();
}
