use std::io::Cursor;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufWriter};
use bytes::{Buf, BytesMut};
use tokio::net::TcpStream;
use mini_redis::{Frame, Result};


struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    cursor: usize,
}


/*
enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null
    Array(Vec<Frame>),
}
*/

// maintain a cursor tracking how much data has been buffered
//

impl Connection {

    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            // allocate buffer with 4kb capacity
            buffer: BytesMut::with_capacity(4096),
            cursor: 0,
        }
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        // Check whether a full frame is available
        match Frame::check(&mut buf) {
            Ok(_) => {
                // Get the byte length of the frame
                let len = buf.position() as usize;

                // Reset the internal cursor for the
                // call to `parse`.
                buf.set_position(0);

                // Parse the frame
                let frame = Frame::parse(&mut buf)?;

                // Discard the frame from the buffer
                self.buffer.advance(len);

                // Return the frame to the caller.
                Ok(Some(frame))
            }
            // Not enough data has been buffered
            Err(Incomplete) => Ok(None),
            // An error was encountered
            Err(e) => Err(e.into()),
        }
    }

    // waits for an entire frame to be received before returning
    // a singl call to tcp read will return x amount of data
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // return the frame if enough data is buffered ....
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }


            // ensure the buffer has cap
            if self.buffer.len() == self.cursor {
                // grow the buffer
                self.buffer.resize(self.cursor * 2, 0)
            }

            // track the number of bytes read into the buffer
            let n = self.stream.read(
                &mut self.buffer[self.cursor..]
            ).await?;

            if 0 == n {
                if self.cursor == 0 {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            } else {
                self.cursor += n;
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();
                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!(),
        }

        self.stream.flush().await;

        Ok(())

    }
}

fn main() {
    println!("Hello, world!");
}
