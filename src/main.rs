use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
pub async fn main() {
    // bind listener
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        // accept a new incoming connection
        let (socket, _) = listener.accept().await.unwrap();
        // spawn tasks for each inbound socket
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    // pseudo database
    let mut db = HashMap::new();

    let mut connection = Connection::new(socket);

    // raed a single frame from stream
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => unimplemented!("{:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}
