fn main() {
    if let Err(err) = aurora::run(std::env::args().nth(1)) {
        eprintln!("错误：{err}");
        std::process::exit(1);
    }
}
