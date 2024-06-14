use bytes::Bytes;
use mini_redis::{Connection, Frame};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};

type Db = HashMap<String, Bytes>;
type SharededDb = Arc<Vec<Mutex<Db>>>;

#[tokio::main]
pub async fn main() {
    // bind listener
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // pseudo database
    let n = 10;
    let maps: Vec<Mutex<Db>> =
        (1..=n).map(|_| Mutex::new(HashMap::new())).collect();
    let db: SharededDb = Arc::new(maps);

    loop {
        // accept a new incoming connection
        let (socket, _) = listener.accept().await.unwrap();
        // make a clone of Arc pointer
        let db = db.clone();
        // spawn tasks for each inbound socket
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: SharededDb) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    // raed a single frame from stream
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let n = hash(cmd.key()) % db.len();
                let mut shared = db[n].lock().unwrap();
                shared.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let n = hash(cmd.key()) % db.len();
                let shared = db[n].lock().unwrap();
                if let Some(value) = shared.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => unimplemented!("{:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}

fn hash(s: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);

    hasher.finish() as usize
}
