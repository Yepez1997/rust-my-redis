use tokio::io;
use tokio::net::TcpListener;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

// async read + asycn write used for utils ...
#[tokio::main]
async fn main() ->{
    let mut listener = TcpListener::bind("127.0.0.1:6142").await.unwrap();

    // reader + writer must be split using io:split
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (mut rd, mut wr) = socket.split();

            if io::copy(&mut, &mut wr).await.is_err() {
                eprintln!("failed to copy");
            }
        })
    }
}
