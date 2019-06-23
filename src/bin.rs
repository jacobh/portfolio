use clap::{self, App, Arg};

fn main() {
    let matches = App::new("Portfolio")
        .version("0.1")
        .author("Jacob Haslehurst <jacob@haslehurst.net>")
        .arg(Arg::with_name("symbol").required(true))
        .get_matches();

    let symbol = matches.value_of("symbol").unwrap();

    println!("symbol: {}", symbol);
}
