use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

const addresses: [&str; 2] = [
    "127.0.0.1:1234",
    "127.0.0.1:1235"
];

#[tokio::main(flavor = "current_thread")]
async fn main() {
    futures::future::try_join_all(addresses.iter().map(|addr| listen_address(addr))).await.expect("Error listening traffic");
}

async fn listen_address(addr: impl ToSocketAddrs) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _socket) = listener.accept().await?;
        tokio::spawn(async move {
            let res = redirect_stream(stream, "127.0.0.1:80").await;
        });
    }
}

async fn redirect_stream(incoming: TcpStream, addr: impl ToSocketAddrs) -> anyhow::Result<()> {
    let outgoing = TcpStream::connect(addr).await?;
    let (mut in_reader, mut in_writer) = incoming.into_split();
    let (mut out_reader, mut out_writer) = outgoing.into_split();
    futures::future::try_join(
    tokio::io::copy(&mut in_reader, &mut out_writer),
    tokio::io::copy(&mut out_reader, &mut in_writer),
    ).await?;
    Ok(())
}
