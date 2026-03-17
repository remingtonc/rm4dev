fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(rm4dev::run(std::env::args()) as u8)
}
