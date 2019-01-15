extern crate protoc_rust_grpc;

fn main() {
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/proto",
        input: &["protos/baacup.proto"],
        includes: &["protos"],
        rust_protobuf: true, // also generate protobuf messages, not just services
        ..Default::default()
    }).expect("protoc-rust-grpc");
}
