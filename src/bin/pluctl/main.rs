use clap::Parser;
use hyper::Client;
use hyperlocal::{UnixClientExt, Uri};
use crate::args::{Commands, PluCtlArgs};

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let args: PluCtlArgs = PluCtlArgs::parse();

    let client = Client::unix();
    match args.subcommand {
        Commands::List => {
            let res = client.get(Uri::new(args.path, "/list").into()).await?;
            println!("{res:?}");
        }
    }
    Ok(())
}