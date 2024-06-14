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
    let mut connection = Connection::new(socket);

    // raed a single frame from stream
    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);

        let response = Frame::Error("unimplemented!".to_string());
        connection.write_frame(&response).await.unwrap();
    }
}
