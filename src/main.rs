use actix_web::{get, App, HttpServer, Responder, HttpResponse, Result};
use serial::SerialPort;
use std::io::{self, prelude::*};
use std::time::SystemTime;
use serde::{Serialize,Deserialize};
use std::env;

const DEVICE_PATH: &str = "/dev/ttyACM0";

#[derive(Serialize,Deserialize,Debug)]
struct UDCO2SStat {
    co2ppm: i32,
    humidity: f32,
    temperature: f32,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Log{
    time: i64,
    status: UDCO2SStat
}

impl UDCO2SStat {
    fn new(co2ppm: i32, humidity: f32, temperature: f32) -> Self {
        UDCO2SStat {
            co2ppm,
            humidity,
            temperature,
        }
    }
}

pub struct UDCO2S {
    dev: String,
}

#[get("/all")]
async fn all() -> impl Responder {
    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            HttpResponse::Ok().json(log)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[get("/co2")]
async fn co2() -> impl Responder {
    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            HttpResponse::Ok().body(log.status.co2ppm.to_string())
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[get("/hum")]
async fn hum() -> impl Responder {
    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            HttpResponse::Ok().body(log.status.humidity.to_string())
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[get("/tmp")]
async fn tmp() -> impl Responder {
    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            HttpResponse::Ok().body(log.status.temperature.to_string())
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let port_result: Result<u16,_> = env::var("VITE_PORT").unwrap().parse();
    match port_result{
        Ok(port) => {
                HttpServer::new(|| {
                    App::new()
                        .wrap(actix_web::middleware::Logger::default())
                        .service(actix_files::Files::new("/assets", "./static/assets").show_files_listing())
                        .service(actix_files::Files::new("/static", "./static").index_file("index.html"))
                        .service(all).service(co2).service(hum).service(tmp)
                })
            .bind((env::var("VITE_LOCAL_ADDRESS").unwrap(), port))?
            .run()
            .await
        },
        Err(_) => {
            println!("PORT is not a appropriate number");
            std::process::exit(1);
        }
    }
}

impl UDCO2S {
    pub fn new(dev: &str) -> Self {
        UDCO2S {
            dev: dev.into(),
        }
    }
    
    pub fn start_logging(&self) -> io::Result<Log> {
        let regex = regex::Regex::new(r"CO2=(?P<co2>\d+),HUM=(?P<hum>\d+\.\d+),TMP=(?P<tmp>-?\d+\.\d+)").unwrap();
        
        let mut port = serial::open(&self.dev).unwrap();

        let option_func = &|settings: &mut dyn serial::SerialPortSettings|{
            _ = settings.set_baud_rate(serial::Baud115200);
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        };

        _ = port.reconfigure(option_func);
        _ = port.set_timeout(std::time::Duration::from_secs(6));
        
        write!(&mut port, "STA\r\n")?;
        print!("{}", read_until(&mut port, '\n')?); // Print the first line
        
        if let Ok(line) = read_until(&mut port, '\n') {
            if let Some(m) = regex.captures(&line) {
                let time_now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64;

                let obj=Log{
                    time : time_now,
                    status:UDCO2SStat::new(
                        m["co2"].parse::<i32>().unwrap(),
                        m["hum"].parse::<f32>().unwrap(),
                        m["tmp"].parse::<f32>().unwrap()
                )};

                write!(&mut port, "STP\r\n")?;
                return Ok(obj);
                // return Ok(m["co2"].parse::<i32>().unwrap().to_string());
            }
        }
        
        write!(&mut port, "STP\r\n")?;

        Err(io::Error::new(
            io::ErrorKind::Other,
            "Could not read from device",
        ))
    }
}

fn read_until(port: &mut dyn serial::SerialPort, del: char) -> io::Result<String> {
    let mut res = String::new();
    loop {
        let mut buf = [0u8; 1];
        match port.read_exact(&mut buf) {
            Ok(_) => {
                let ch = char::from(buf[0]);
                if ch == del {
                    return Ok(res);
                } else {
                    res.push(ch);
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut => return Ok(res),
                _ => return Err(e.into()),
            },
        }
    }
}