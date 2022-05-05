extern crate bytes;
extern crate num_derive;
extern crate num_traits;
extern crate thiserror;

use pica::{web, Pica, PicaCommand};
use std::path::PathBuf;

use anyhow::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio::try_join;

const UCI_PORT: u16 = 7000;

async fn accept_incoming(tx: mpsc::Sender<PicaCommand>) -> Result<()> {
    let uci_socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, UCI_PORT);
    let uci_listener = TcpListener::bind(uci_socket).await?;
    println!("Pica: Listening on: {}", UCI_PORT);

    loop {
        let (socket, addr) = uci_listener.accept().await?;
        println!("Uwb host addr: {}", addr);
        tx.send(PicaCommand::Connect(socket)).await?
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "pica", about = "Virtual UWB subsystem")]
struct Opts {
    /// Output directory for storing .pcapng traces.
    /// If provided, .pcapng traces of client connections are automatically
    /// saved under the name `device-{handle}.pcapng`.
    #[structopt(short, long, parse(from_os_str))]
    pcapng_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::from_args();
    let (event_tx, _) = broadcast::channel(16);

    let mut pica = Pica::new(event_tx.clone(), opts.pcapng_dir);
    let tx = pica.tx();

    try_join!(
        accept_incoming(tx.clone()),
        pica.run(),
        web::serve(tx, event_tx)
    )?;

    Ok(())
}
