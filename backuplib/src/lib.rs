pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn print_hello() {
    println!("Hello World!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
