use clap::Parser;
fn main() {
    match bestls::CLI::parse().run() {
        Ok((_, output)) => {
            println!("{}", output);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
