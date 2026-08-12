use clap::Parser;
fn main() {
    match bestls::CLI::parse().run() {
        Ok(output) => {
            println!("{}", output);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
