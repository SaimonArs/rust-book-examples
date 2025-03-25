mod data {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}


mod utils {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use prost_types::Timestamp;
    use crate::data::Data;
    use prost::Message;

    pub fn protobuf_timestamp_to_naive_date_time(timestamp: Timestamp) -> NaiveDateTime {
        let datetime_utc: DateTime<Utc> = DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32).unwrap();
        datetime_utc.naive_utc()
    }

    pub fn vec_to_data_proto(mut vec: Vec<u8>) -> Result<Data, ()> {
        if vec.len() > 4 {
            let mut vec_len: usize = 0 ;
            for i in (0..4).rev() {
                vec_len = vec_len << 8;
                vec_len += vec[i] as usize;
            }
            vec = vec[4..].to_owned();

            if vec_len <= vec.len() {
                return match Data::decode(&vec[..vec_len]) {
                    Ok(r) => Ok(r),
                    Err(_) => Err(())
                }
            }
        }
        Err(())
    }
}

pub mod db_client {
    use postgres::{Client, NoTls};
    use crate::data::Data;
    use crate::utils::protobuf_timestamp_to_naive_date_time;

    pub struct Database {
        postgres_client: Client,
    }

    impl Database {
        pub fn build(postgres_url: String) -> Database {
            loop {
                match Client::connect(&postgres_url, NoTls) {
                    Ok(c) => {return Database {postgres_client: c}},
                    Err(_) => {continue;}
                }
            }
        }

        pub fn init_table(&mut self) {
            self.postgres_client.batch_execute("
            CREATE TABLE IF NOT EXISTS data (
            id              SERIAL PRIMARY KEY,
            device_id       OID NOT NULL,
            event_id        OID NOT NULL,
            humidity        real NOT NULL,
            temperature     real NOT NULL,
            read_time       timestamp NOT NULL
            )").unwrap();
        }

        pub fn add_data(&mut self, data: Data) {
            let read_time = protobuf_timestamp_to_naive_date_time(data.read_time.unwrap());
            self.postgres_client.execute(
                "INSERT INTO data 
                (device_id, event_id, humidity, temperature, read_time) 
                VALUES 
                ($1, $2, $3, $4, $5)",
                &[&data.device_id, &(data.event_id as u32), &data.humidity, &data.temperature, &read_time]
                ).unwrap();

        }
    }
}

pub mod listener {
    use std::{io::{BufReader, Read}, net::TcpListener};

    use crate::{db_client::Database, utils::vec_to_data_proto};
    pub struct Listener {
        listener: TcpListener,
        db: Database
    }

    impl Listener {
        pub fn build(db: Database,addr: String, port: String) -> Listener {
            Listener {
                listener: TcpListener::bind(addr+":"+&port).unwrap(),
                db: db
            }
        }

        pub fn start(&mut self) {
            for stream in self.listener.incoming() {
                let stream = stream.unwrap();
                let mut buf_reader = BufReader::new(&stream);
                let mut vec_data = Vec::new();
                buf_reader.read_to_end(&mut vec_data).unwrap();
                let data =  match vec_to_data_proto(vec_data) {
                    Ok(d) => d,
                    Err(_) => continue
                };
                self.db.add_data(data);

            }
        }
    }
}