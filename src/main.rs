#![warn(rust_2018_idioms)]

use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use rlimit::{Resource, getrlimit};
use chashmap::{CHashMap};

use std::env;
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    set_rlimit().await?;
    let chmap:CHashMap<u16, u8> = CHashMap::new();
    println!("rlimit: {:?}", getrlimit(Resource::NOFILE).unwrap());
    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:12345".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let mut listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        println!("accept from {:?}", socket.peer_addr());
        let rport = socket.peer_addr().unwrap().port();
        println!("remote port: {:?}", rport);
        if chmap.contains_key(&rport) {
            println!("add key {:?} exists", rport);
        } else {
            chmap.insert_new(rport, 1);
        }
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}

async fn set_rlimit() -> std::io::Result<()> {
  Resource::NOFILE.set(1048576, 1048576)
}
