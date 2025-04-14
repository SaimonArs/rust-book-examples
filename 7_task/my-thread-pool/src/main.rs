use my_thread_pool::ThreadPool;
use std::{
    env, fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicUsize},
    thread,
    time::Duration,
};

fn main() {
    // _web_server();
    _recursive_file_count();
}

fn _recursive_file_count() {
    let mut args = env::args();
    args.next();
    let a = args.next().unwrap();

    let counter = Arc::new(AtomicUsize::new(0));

    let counter_clone = counter.clone();

    let pool = Arc::new(ThreadPool::new(8).unwrap());

    _handle_count_file(
        counter_clone,
        Path::new(a.as_str()).to_owned(),
        Arc::clone(&pool),
    );

    loop {
        if Arc::strong_count(&pool) == 1 {
            break;
        }
    }
    println!("{}", counter.load(std::sync::atomic::Ordering::SeqCst));
}

fn _handle_count_file(counter: Arc<AtomicUsize>, p: PathBuf, pool: Arc<ThreadPool>) {
    let rdi = match fs::read_dir(p) {
        Ok(i) => i,
        Err(_) => return,
    };

    for i in rdi {
        let dir_entry = match i {
            Ok(de) => de,
            Err(_) => continue,
        };

        let path_buffer = dir_entry.path();

        if path_buffer.is_dir() && !path_buffer.is_symlink() {
            let counter_clone = counter.clone();
            let pool_clone = Arc::clone(&pool);
            let _ = pool
                .execute(move || {
                    _handle_count_file(counter_clone, path_buffer, pool_clone);
                })
                .unwrap();
        } else {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

fn _web_server() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let _ = pool.execute(|| {
            _handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn _handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
