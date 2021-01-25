use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};



// async read and async write allow to read and write bytes asycn
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    let mut buffer = Vec::new();


    // ::read -> reads specified bytes
    // ::read_to_end -> reads until eof end of file ie read the whole file
    let n = f.read_to_end(&mut buffer).await?;

    println!("The bytes: {:?}", &buffer[..n]);
    Ok(()) // ok((0)) signifies that the stream is closed
}
