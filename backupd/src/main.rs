mod server;
mod storage;
mod configuration;

use std::env;
use std::thread;

use server::BaacupImpl;
use backuplib::grpc::ServerBuilder;
use backuplib::rpc::BaacupServer;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    backuplib::print_hello();
    println!("backupd v{} using backuplib v{}", VERSION, backuplib::VERSION);

    let filename = env::args().skip(1).next().unwrap_or("backup/".into());

    let mut server_builder = ServerBuilder::new_plain();
    server_builder.http.set_port(8000);
    let baacup_impl = BaacupImpl::new(filename);
    server_builder.add_service(BaacupServer::new_service_def(baacup_impl));
    let _server = server_builder.build().unwrap();

    loop {
        thread::park();
    }
}
