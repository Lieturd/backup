const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    backuplib::print_hello();
    println!("backupd v{} using backuplib v{}", VERSION, backuplib::VERSION);
}
