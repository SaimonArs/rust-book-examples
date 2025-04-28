mod info {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}
use chrono::Local;
use egui::{Image, Widget as _};
use info::Info;
use prost::Message;

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex, atomic::AtomicBool, mpsc},
    thread,
    time::{Duration, Instant},
};

use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};

use eframe::egui;

use bytemuck::{Pod, Zeroable};

fn main() {
    ffmpeg_sidecar::download::auto_download().unwrap();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Video server",
        native_options,
        Box::new(|cc| Ok(Box::new(Server::new(cc)))),
    )
    .unwrap();
}

#[derive(Debug, Clone, Copy)]
struct Robot {
    addr: SocketAddr,
    x: f32,
    y: f32,
}

fn notifier(vec_robot: Arc<Mutex<HashMap<u32, Robot>>>) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let vec_clone = Arc::clone(&vec_robot);
        pool.spawn(|| {
            notifier_handler(stream, vec_clone);
        });
    }
}

fn notifier_handler(mut stream: TcpStream, vec_robot: Arc<Mutex<HashMap<u32, Robot>>>) {
    let mut vec_data = Vec::new();
    match stream.read_to_end(&mut vec_data) {
        Err(_) => {
            return;
        }
        _ => {}
    };
    let data = Info::decode(&vec_data[..]).unwrap();
    let mut addr = stream.peer_addr().unwrap();
    addr.set_port(data.port as u16);
    let r = Robot {
        addr,
        x: data.x,
        y: data.y,
    };
    log(data.id, r.clone());
    vec_robot.lock().unwrap().insert(data.id, r);
}

fn log(id: u32, r: Robot) {
    let mut path = id.to_string();
    path.push_str(".log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    let log_entry = format!(
        "[{}] id: {}, ip:port: {}, x: {}, y: {} \n",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        id,
        r.addr,
        r.x,
        r.y
    );

    file.write_all(log_entry.as_bytes()).unwrap();
}

fn video_provider(
    frame_tx: mpsc::Sender<egui::ColorImage>,
    addr_rx: mpsc::Receiver<SocketAddr>,
    ctx: egui::Context,
) {
    let drop_video_rc = Arc::new(AtomicBool::new(false));
    for recv in addr_rx {
        drop_video_rc.fetch_not(std::sync::atomic::Ordering::SeqCst);
        thread::sleep(Duration::from_millis(200));
        drop_video_rc.fetch_not(std::sync::atomic::Ordering::SeqCst);

        let clone_fd = Arc::clone(&drop_video_rc);
        let clone_ftx = frame_tx.clone();
        let clone_ctx = ctx.clone();
        thread::spawn(move || video_rc(recv, clone_ftx, clone_ctx, clone_fd));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RgbPixel {
    r: u8,
    g: u8,
    b: u8,
}

fn video_rc(
    addr: SocketAddr,
    frame_tx: mpsc::Sender<egui::ColorImage>,
    ctx: egui::Context,
    drop_flag: Arc<AtomicBool>,
) {
    if let Ok(stream) = TcpStream::connect(addr) {
        let mut instance = FfmpegCommand::new()
            .format("mpegts")
            .input("-")
            .codec_video("rawvideo")
            .pix_fmt("rgb24")
            .format("rawvideo")
            .args(["-r", "30"])
            .output("-")
            .spawn()
            .unwrap();

        let mut stdin = instance.take_stdin().unwrap();
        let cf = Arc::clone(&drop_flag);
        thread::spawn(move || {
            let mut buffer = [0u8; 188 * 300];
            // let mut inst = Instant::now();
            // let mut acc = 0;
            loop {
                let n = (&stream).read(&mut buffer).unwrap();
                // acc += n;
                stdin.write_all(&buffer[..n]).unwrap();
                if cf.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                // if inst.elapsed().as_millis() > 1000 {
                //     eprintln!("speed: {:#?} b/s", acc);
                //     acc = 0;
                //     inst = Instant::now();
                // }
            }
        });

        let mut instance_iter = instance.iter().unwrap();
        let mut inst = Instant::now();
        while let Some(event) = instance_iter.next() {
            if drop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            match event {
                FfmpegEvent::OutputFrame(frame) => {
                    let pixels: &[RgbPixel] = bytemuck::cast_slice(&frame.data);
                    let pixels = pixels
                        .iter()
                        .map(|p| egui::Color32::from_rgb(p.r, p.g, p.b))
                        .collect();

                    let image = egui::ColorImage {
                        size: [frame.width as usize, frame.height as usize],
                        pixels,
                    };

                    frame_tx.send(image).unwrap();
                    loop {
                        if inst.elapsed().as_millis() > 33 {
                            break;
                        }
                    }
                    ctx.request_repaint();
                    // println!("framerate: {:#?}", inst.elapsed().as_millis());
                    inst = Instant::now();
                }
                _ => {}
            }
        }
    }
}

struct Server {
    frame_receiver: mpsc::Receiver<egui::ColorImage>,
    current_frame: Option<(Arc<egui::ColorImage>, egui::TextureHandle)>,
    addr_tx: mpsc::Sender<SocketAddr>,
    selected_key: Option<u32>,
    hm_robots: Arc<Mutex<HashMap<u32, Robot>>>,
}

impl Server {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let hm_robots = Arc::new(Mutex::new(HashMap::new()));
        let hm_clone = Arc::clone(&hm_robots);
        thread::spawn(move || {
            notifier(hm_clone);
        });

        let (tx, rx) = mpsc::channel();
        let (addr_tx, addr_rx) = mpsc::channel();
        let ctx = cc.egui_ctx.clone();

        thread::spawn(move || {
            video_provider(tx, addr_rx, ctx);
        });

        Self {
            frame_receiver: rx,
            current_frame: None,
            addr_tx,
            selected_key: None,
            hm_robots,
        }
    }
}

impl eframe::App for Server {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(frame) = self.frame_receiver.try_recv() {
                self.current_frame = Some((
                    Arc::new(frame.clone()),
                    ui.ctx().load_texture("frame", frame, Default::default()),
                ))
            }

            ui.centered_and_justified(|ui| {
                if let Some((_, cf)) = &self.current_frame {
                    Image::new(cf).shrink_to_fit().ui(ui);
                } else {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.set_height(100.0);
                        ui.centered_and_justified(|ui| {
                            ui.label("No video");
                        });
                    });
                }
            });
        });

        egui::SidePanel::left("side")
            .frame(egui::Frame {
                fill: egui::Color32::from_rgba_unmultiplied(30, 30, 30, 70),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Robots:");
                    ui.separator();
                    for (key, val) in self.hm_robots.lock().unwrap().iter() {
                        let f = format!(
                            "{:#?}, x: {:#?}, y: {:#?}, addr: {:#?}",
                            key, val.x, val.y, val.addr
                        );

                        let mut btn = egui::Button::new(f).frame(false);

                        if Some(key) == self.selected_key.as_ref() {
                            btn = btn.fill(egui::Color32::from_rgb(50, 60, 70));
                        }

                        if ui.add(btn).clicked() {
                            self.selected_key = Some(key.clone());
                            self.addr_tx.send(val.addr).unwrap();
                        }
                    }
                })
            });
    }
}
