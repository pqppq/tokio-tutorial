use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        // 応答用のoneshot sender
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        // 応答用のoneshot sender
        resp: Responder<()>,
    },
}

#[tokio::main]
pub async fn main() {
    let cap = 32;
    let (sender, mut reciever) = mpsc::channel::<Command>(cap);
    let sender2 = sender.clone();

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = reciever.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp } => {
                    let res = client.get(&key).await;
                    resp.send(res).unwrap();
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    resp.send(res).unwrap();
                }
            }
        }
    });

    let task1 = tokio::spawn(async move {
        let (resp_sender, resp_receiver) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_sender,
        };
        sender.send(cmd).await.unwrap();

        let res = resp_receiver.await;
        println!("GOT = {:?}", res);
    });

    let task2 = tokio::spawn(async move {
        let (resp_sender, resp_receiver) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_sender,
        };
        sender2.send(cmd).await.unwrap();

        let res = resp_receiver.await;
        println!("GOT = {:?}", res);
    });

    task1.await.unwrap();
    task2.await.unwrap();
    manager.await.unwrap();
}
