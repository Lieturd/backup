extern crate protoc_rust_grpc;

fn main() {
    let input_files = ["protos/baacup.proto"];
    for filename in input_files.iter() {
        println!("cargo:rerun-if-changed={}", filename);
    }

    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/proto",
        input: &input_files,
        includes: &["protos"],
        rust_protobuf: true, // also generate protobuf messages, not just services
        ..Default::default()
    }).expect("protoc-rust-grpc");
}
