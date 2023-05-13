use clap::Parser;
use hyper::Client;
use hyperlocal::{UnixClientExt, Uri};
use port_plumber::api::Endpoint;
use crate::args::{Commands, PluCtlArgs};
use crate::client::SimpleRest;

mod args;
mod client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args: PluCtlArgs = PluCtlArgs::parse();

    let client = SimpleRest::from(Client::unix());
    match args.subcommand {
        Commands::List => {
            let res: Vec<Endpoint> = client.get(Uri::new(args.path, "/list")).await?;
            println!("{res:?}");
        },
        Commands::Resolve { name } => {
            let res: Endpoint = client.get(Uri::new(args.path, &format!("/resolve/{name}"))).await?;
            println!("{}", res.ip);
        }
    }
    Ok(())
}

