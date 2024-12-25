use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use mio::net::{TcpListener, TcpStream};
use tungstenite::{accept, WebSocket};

use super::time::HourPassedEvent;

pub struct DataCollectionPlugin;

impl Plugin for DataCollectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                initialize_data_collection_directory,
                initialize_websocket_server,
            ),
        )
        .add_systems(
            Update,
            try_connecting_to_websocket.run_if(on_event::<HourPassedEvent>),
        );
    }
}

#[derive(Resource)]
pub struct DataCollectionDirectory(pub PathBuf);

fn initialize_data_collection_directory(mut cmd: Commands) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let dir_name = format!("simulation_{}", timestamp);
    let path = Path::new("data").join(dir_name);

    std::fs::create_dir_all(&path).expect("Can't ensure path to data collection directory exists");

    cmd.insert_resource(DataCollectionDirectory(path));
}

#[derive(Resource)]
pub struct WebsocketServer {
    server: TcpListener,
    pub websocket: Option<WebSocket<TcpStream>>,
}

impl WebsocketServer {
    pub fn remove_websocket_if_its_dead(&mut self) {
        if let Some(websocket) = self.websocket.as_mut() {
            let ping_result = websocket.send(tungstenite::Message::Ping(tungstenite::Bytes::new()));

            if ping_result.is_err() {
                self.websocket = None;
            }
        }
    }
}

fn initialize_websocket_server(mut cmd: Commands) {
    let socket_address = "127.0.0.1:42069"
        .parse()
        .expect("Failed to parse websocket address");

    let server = TcpListener::bind(socket_address).expect("Failed to setup websocket server");

    cmd.insert_resource(WebsocketServer {
        server,
        websocket: None,
    });
}

fn try_connecting_to_websocket(mut server: ResMut<WebsocketServer>) {
    server.remove_websocket_if_its_dead();

    if server.websocket.is_some() {
        return;
    }

    match server.server.accept() {
        Ok((stream, _)) => {
            println!("Got new connection");

            match accept(stream) {
                Ok(websocket) => server.websocket = Some(websocket),
                Err(e) => {
                    println!("Failed to accept WebSocket connection");
                    dbg!(e);
                }
            }
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => {
            println!("Would block, not connecting to websocket")
        }
        Err(_) => println!("Some kind of error while searching for websocket connections"),
    };
}
