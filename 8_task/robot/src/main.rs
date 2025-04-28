mod info {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}
use info::Info;
use prost::Message;
use serde::Deserialize;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ffmpeg_sidecar::command::FfmpegCommand;

#[derive(Deserialize, Debug)]
struct State {
    id: u32,
    x: f32,
    y: f32,
    addr: String,
    port: u32,
    video: String,
}

fn read_config() -> State {
    let content = std::fs::read_to_string("conf.toml").unwrap();
    let conf: State = toml::from_str(&content).unwrap();
    conf
}

fn notify_server(state: &Arc<Mutex<State>>) {
    let s = state.lock().unwrap();
    // dbg!(&s);
    let mut stream = match TcpStream::connect(s.addr.clone()) {
        Ok(s) => {
            println!("Success");
            s
        }
        Err(_) => {
            println!("err!");
            return ();
        }
    };

    let info = Info {
        id: s.id,
        x: s.x,
        y: s.y,
        port: s.port.clone(),
    };

    let vec_data = info.encode_to_vec();
    stream.write_all(&vec_data).unwrap();
}

fn stream_video(mut stream: TcpStream, video_path: String) {
    let mut input_command = "-stream_loop -1 ".to_string();

    if video_path.as_str() == "testsrc" {
        input_command = "-f lavfi -i testsrc=size=2k:rate=30:duration=43200".to_string();
    } else {
        input_command += &video_path;
    }
    // dbg!(&input_command);

    let mut instance = FfmpegCommand::new()
        .args(input_command.as_str().split(' '))
        .args("-flags +low_delay -preset:v ultrafast -tune zerolatency -threads 4".split(' '))
        .args(["-c:v", "libx264"])
        .args(["-r", "30"])
        .format("mpegts")
        .args(["-b:v", "3M"])
        .output("-")
        .spawn()
        .unwrap();

    let mut stdout = instance.take_stdout().unwrap();

    let mut buf = [0u8; 188 * 300];
    loop {
        let bytes_read = stdout.read(&mut buf).unwrap();

        match stream.write_all(&buf[..bytes_read]) {
            Ok(_) => {}
            Err(_) => {
                break;
            }
        };
    }
}

fn main() {
    ffmpeg_sidecar::download::auto_download().unwrap();
    let state = Arc::new(Mutex::new(read_config()));
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            state_clone.lock().unwrap().x += 0.001;
            state_clone.lock().unwrap().y -= 0.001;
            notify_server(&state_clone);
            thread::sleep(Duration::from_secs(2));
        }
    });

    let listener = TcpListener::bind(SocketAddr::from((
        [0, 0, 0, 0],
        state.lock().unwrap().port as u16,
    )))
    .unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let video_path = state.lock().unwrap().video.clone();
        stream_video(stream, video_path);
    }
}
