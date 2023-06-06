extern crate getopts;
use getopts::Options;

use std::env;
use std::io;

use std::io::{Error, ErrorKind};
use rand::Rng;
use std::thread;
use std::slice;
use std::mem;
use chrono::{Datelike, Timelike, Utc};
use std::time::{Duration, Instant};
mod msp;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
#[repr(packed)]
struct GpsSol {
    instance: u8,
    gpsweek: u16,
    mstow: u32,
    fixtype: u8,
    satellitesinview: u8,
    horizontalposaccuracy: u16,
    verticalposaccuracy: u16,
    horizontalvelaccuracy: u16,
    hdop: u16,
    longitude: i32,
    latitude: i32,
    mslaltitude: i32,
    nedvelnorth: i32,
    nedveleast: i32,
    nedveldown: i32,
    groundcourse: u16,
    trueyaw: u16,
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    min: u8,
    sec: u8,
}

#[derive(Default)]
#[repr(packed)]
struct RawGps {
    fix: u8,
    numsat: u8,
    lat: i32,
    lon: i32,
    alt: i16,
    gspd: u16,
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!(
        "Usage: {} [options] device-node\nVersion: {}",
        program, VERSION
    );
    print!("{}", opts.usage(&brief));
}

fn main() ->  io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("v", "version", "Show version");
    opts.optflag("s", "sensor", "Use MSP2_SENSOR_GPS (vice MSP_SET_RAW GPS)");
    opts.optflag("h", "help", "print this help menu");

   let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return Ok(());
    }

    if matches.opt_present("v") {
        println!("{}", VERSION);
        return Ok(());
    }

    let use_sensor: bool =  matches.opt_present("s");

    let defdev = if !matches.free.is_empty() {
        &matches.free[0]
    } else {
	return Err(Error::new(ErrorKind::Other, "Device node missing"));
    };

    let  mut msd = msp::MSPDev::new(&defdev)?;
    let mut cmd = msp::MSG_API_VERSION;
    loop {
	match msd.send_msp(cmd,&[]) {
	    Ok(v) => {
		if v.ok == msp::MSPRes::Ok {

		    cmd = match msp::describe(&v) {
			Some(c) => c,
			None => {
			    break
			}
		    };
		} else {
		    println!("Read {:?}", v);
		    return Ok(())
		}
	    },
	    Err(e) => return Err(e),
	}
    }

    let mut lat: i32 = 357610000;
    let mut lon: i32 = 1403789450;
    let fix:u8 = 3;
    let mut numsat:u8 = 13;
    let mut alt:i16 = 100;
    let gspd:u16 = 10;
    let gcse:u16 = 247;

    let mut rng = rand::thread_rng();
    let s: &[u8];
    let mut g = GpsSol::default();
    let mut r = RawGps::default();

    if use_sensor {
	let p: *const GpsSol = &g;
        let p: *const u8 = p as *const u8;
        s = unsafe {
            slice::from_raw_parts(p, mem::size_of::<GpsSol>())
        };
	g.instance = 1;
        g.gpsweek = 0xffff;
        g.mstow = 0;
        g.fixtype = fix;
        g.horizontalposaccuracy = 50;
        g.verticalposaccuracy = 50;
        g.horizontalvelaccuracy = 0;
        g.hdop = 100;
        g.nedvelnorth = 0;
        g.nedveleast = 0;
        g.nedveldown = 0;
        g.trueyaw = 0xffff;
	println!("Message: MSP2_SENSOR_GPS");
    } else {
	let p: *const RawGps = &r;
        let p: *const u8 = p as *const u8;
        s = unsafe {
            slice::from_raw_parts(p, mem::size_of::<RawGps>())
        };
	r.fix = fix;
	println!("Message: MSP_SET_RAW_GPS");
    }

    let mut start = Instant::now();

    loop {
	if use_sensor {
            let now = Utc::now();
            g.satellitesinview = numsat;
            g.year = now.year() as u16;
            g.month = now.month() as u8;
            g.day = now.day() as u8;
            g.hour = now.hour() as u8;
            g.min = now.minute() as u8;
            g.sec = now.second() as u8;
            g.latitude = lat;
            g.longitude = lon;
            g.mslaltitude = (alt*100) as i32;
	    g.groundcourse = gcse*100;
	    let (vx,vy) = polar2cartesian(gspd as f64, gcse as f64);
            g.nedveleast = (vx * 100.0) as i32;
	    g.nedvelnorth = (vy * 100.0) as i32;
	    msd.write_msp(msp::MSP2_SENSOR_GPS, s)?;
	} else {
	    r.numsat = numsat;
            r.lat =  lat;
            r.lon = lon;
            r.alt = alt;
            r.gspd = gspd*100;
	    match msd.send_msp(msp::MSP_SET_RAW_GPS, s) {
		Ok(_v) => (), //println!("{:?}", _v),
		Err(e) => return Err(e),
	    }
	}

	thread::sleep(Duration::from_millis(200));
	if start.elapsed() > Duration::new(2,0) {
	    numsat = rng.gen_range(10..27);
	    alt = rng.gen_range(10..120);
	    lat += rng.gen_range(-1000..1000);
	    lon += rng.gen_range(-1000..1000);
	    start = Instant::now();
	}
    }
}

fn polar2cartesian(r: f64, theta: f64) -> (f64, f64) {
    let theta = theta.to_radians();
    let x = r*theta.cos();
    let y = r*theta.sin();
    (x,y)
}
