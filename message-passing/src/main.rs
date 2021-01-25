use mini_redis::client;
use bytes::Bytes;
use tokio::sync::mpsc;
use tokio::sync::oneshot; // single producer single consuemr used to opt for sending one value

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Vec<u8>,
        resp: Responder<()>,
    }
}


type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let (tx, mut rx) = mpsc::channel(32);

    let tx2 = tx.clone(); // use another producer


    // move rx ownership into the thread
    // proccesse messages from the channel
    let manager = tokio::spawn(async move {

        // connect to mini redis client
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // receive messages ....
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val.into()).await;
                    let _ = resp.send(res);
                }
            }
        }
    });



    // t1 and t2 return a join handler
    let t1 = tokio::spawn(async move {
        // oneshot channel -> https://docs.rs/tokio/0.1.18/tokio/sync/oneshot/fn.channel.html
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Get {
            key: "hello".to_string(),
            resp: resp_tx,
        };

        // send the command
        tx.send(cmd).await.unwrap();

        // wait for the command with oneshot channel
        let res = resp_rx.await;
        println!("GOT = {:?}", res);

    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: b"bar".to_vec(),
            resp: resp_tx,
        };

        tx2.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("GOT = {:?}", res);


    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();

}
