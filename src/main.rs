fn main() {
    if let Err(err) = aurora::run_args(std::env::args().skip(1)) {
        eprintln!("错误：{err}");
        std::process::exit(1);
    }
}
