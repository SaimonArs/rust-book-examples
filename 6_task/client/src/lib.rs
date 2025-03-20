mod data {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}
mod dht;

use prost::Message;
use prost_types::Timestamp;
use std::{io::Write, net::TcpStream, time, time::Duration, thread};

pub struct Config {
    device_id: u32,
    timer_sec: u64,
    address: String,
    port: String
}

impl Config {
    pub fn build(device_id: u32, timer_sec: u64, address: String, port: String) -> Self {
        Config { device_id: device_id, timer_sec: timer_sec, address: address, port: port }
    }
    pub fn device_id(&self) -> u32 {
        self.device_id
    }
    pub fn timer_sec(&self) -> u64 {
        self.timer_sec
    }
    pub fn address(&self) -> &String {
        &self.address
    }
    pub fn port(&self) -> &String {
        &self.port
    }
}

pub struct ESP {
    config: Config,
    event_id: u64,
    dht: dht::DHT,
}

impl ESP {
    pub fn build(config: Config) -> Self {
        ESP { config: config, event_id: 0, dht: dht::DHT::build()}
    }

    pub fn start(&mut self) {
        loop {
            thread::sleep(Duration::from_secs(self.config.timer_sec));
            let mut data = data::Data::default();

            data.device_id = self.config.device_id();
            data.event_id = self.event_id;
            self.event_id += 1;

            data.humidity = self.dht.read_humidity();
            data.temperature = self.dht.read_temperature();
        
            let mut read_ts = Timestamp::default();
            let duration = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
            read_ts.seconds = duration.as_secs() as i64;
            read_ts.nanos = duration.subsec_nanos() as i32;

            data.read_time = Some(read_ts);

            let mut stream = match TcpStream::connect(self.config.address().to_owned() + ":" + self.config.port()) {
                Ok(stream) => {println!("Success"); stream},
                Err(_) => {println!("Error"); continue;},
            };

            let vec_data = data.encode_to_vec();
            stream.write_all(&vec_data).unwrap();
            stream.flush().unwrap();

        }
    }
}
