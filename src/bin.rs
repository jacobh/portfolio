use clap::{self, App, Arg, SubCommand};

fn main() {
    let symbol_arg = Arg::with_name("symbol").required(true);

    let matches = App::new("Portfolio")
        .version("0.1")
        .author("Jacob Haslehurst <jacob@haslehurst.net>")
        .subcommand(SubCommand::with_name("latest-price").arg(&symbol_arg))
        .subcommand(SubCommand::with_name("summary").arg(&symbol_arg))
        .get_matches();

    match matches.subcommand() {
        ("latest-price", Some(matches)) => {
            let symbol = matches.value_of("symbol").unwrap();

            let price = portfolio::get_latest_price_for_equity(symbol.into()).unwrap();

            println!("{}: {}", symbol, price);
        }
        ("summary", Some(matches)) => {
            let symbol = matches.value_of("symbol").unwrap();

            let summary =
                portfolio::summary_for_equity(symbol.into(), portfolio::TimePeriod::Year).unwrap();

            println!("{:?}", summary)
        }
        (&_, _) => println!("Command not recognised"),
    };
}
