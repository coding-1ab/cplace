use client::app::Configuration;
use client::run;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut configuration = Configuration::default();
    if let Some(first) = args.get(0) {
        configuration.tiles = first.parse()?;
    };
    run(configuration)
}