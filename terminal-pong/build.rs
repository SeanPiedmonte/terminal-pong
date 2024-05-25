
fn main() {
    protobuf_codegen::Codegen::new()
        .cargo_out_dir("proto")
        .include("proto")
        .input("proto/message.proto")
        .run_from_script();
}
