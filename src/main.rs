use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use serial::SerialPort;
use std::io::{self, prelude::*};
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;
use serde::{Serialize,Deserialize};
use std::env;

const DEVICE_PATH: &str = "/dev/ttyACM0";

#[derive(Serialize,Deserialize,Debug,Clone)]
struct UDCO2SStat {
    co2ppm: i32,
    humidity: f32,
    temperature: f32,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
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

struct SecondsInterval {
    interval: u16,
}

struct LogCache {
    data: Mutex<Log>,
}

pub struct UDCO2S {
    dev: String,
}

#[get("/all")]
async fn all(seconds_interval: web::Data<SecondsInterval>, log_cache: web::Data<LogCache>) -> impl Responder {
    let mut data = log_cache.data.lock().unwrap();
    if should_use_cache(&data, seconds_interval) {
        return HttpResponse::Ok().json(data.clone());
    }

    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            *data = log.clone();
            HttpResponse::Ok().json(log)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[get("/co2")]
async fn co2(seconds_interval: web::Data<SecondsInterval>, log_cache: web::Data<LogCache>) -> impl Responder {
    let mut data = log_cache.data.lock().unwrap();
    if should_use_cache(&data, seconds_interval) {
        return HttpResponse::Ok().body(data.status.co2ppm.to_string());
    }

    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            *data = log.clone();
            HttpResponse::Ok().body(log.status.co2ppm.to_string())
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[get("/hum")]
async fn hum(seconds_interval: web::Data<SecondsInterval>, log_cache: web::Data<LogCache>) -> impl Responder {
    let mut data = log_cache.data.lock().unwrap();
    if should_use_cache(&data, seconds_interval) {
        return HttpResponse::Ok().body(data.status.humidity.to_string());
    }

    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            *data = log.clone();
            HttpResponse::Ok().body(log.status.humidity.to_string())
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}

#[get("/tmp")]
async fn tmp(seconds_interval: web::Data<SecondsInterval>, log_cache: web::Data<LogCache>) -> impl Responder {
    let mut data = log_cache.data.lock().unwrap();
    if should_use_cache(&data, seconds_interval) {
        return HttpResponse::Ok().body(data.status.temperature.to_string());
    }

    let sensor = UDCO2S::new(DEVICE_PATH);
    let log = sensor.start_logging();
    match log{
        Ok(log) => {
            *data = log.clone();
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
    let sconds_interval = web::Data::new(SecondsInterval {
        interval: 4,
    });
    let log_cache = web::Data::new(LogCache {
        data: Mutex::new(Log{
            time: 0,
            status: UDCO2SStat::new(0, 0.0, 0.0)
        }),
    });
    match port_result{
        Ok(port) => {
                HttpServer::new(move || {
                    App::new()
                        .app_data(log_cache.clone())
                        .app_data(sconds_interval.clone())
                        .wrap(actix_web::middleware::Logger::default())
                        .service(actix_files::Files::new("/assets", "./static/assets"))
                        .service(actix_files::Files::new("/graph", "./static").index_file("index.html"))
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
                let obj=Log{
                    time : now_time(),
                    status: UDCO2SStat::new(
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

fn should_use_cache(data: &MutexGuard<'_, Log>, seconds_interval: web::Data<SecondsInterval>) -> bool {
    data.time + i64::from(seconds_interval.interval) > now_time()
}

fn now_time() -> i64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
}