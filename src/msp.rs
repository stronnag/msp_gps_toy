use serial2::SerialPort;
use std::io;
use std::time::Duration;

pub const MSG_NAME: u16 = 10;
pub const MSG_API_VERSION: u16 = 1;
pub const MSG_FC_VARIANT: u16 = 2;
pub const MSG_FC_VERSION: u16 = 3;
pub const MSG_BOARD_INFO: u16 = 4;
pub const MSG_BUILD_INFO: u16 = 5;
pub const MSG_ANALOG: u16 = 110;
pub const MSP_SET_RAW_GPS: u16 = 201;
pub const MSP2_SENSOR_GPS: u16 = 0x1f03;

//pub const MSG_IDENT: u16 = 100;
//pub const MSG_WP_GETINFO: u16 = 20;
//pub const MSG_RAW_GPS: u16 = 106;
//pub const MSG_DEBUGMSG: u16 = 253;
//pub const MSG_STATUS_EX: u16 = 150;
//pub const MSG_INAV_STATUS: u16 = 0x2000;
//pub const MSG_MISC2: u16 = 0x203a;

enum States {
    Init,
    M,
    Dirn,
    Len,
    Cmd,
    Data,
    Crc,

    XHeader2,
    XFlags,
    XId1,
    XId2,
    XLen1,
    XLen2,
    XData,
    XChecksum,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MSPRes {
    Ok,
    Crc,
    Dirn,
    Fail,
}

impl Default for MSPRes {
    fn default() -> Self {
        MSPRes::Fail
    }
}

pub struct MSPDev {
    sd: SerialPort
}

#[derive(Debug, Default, Clone)]
pub struct MSPMsg {
    pub len: u16,
    pub cmd: u16,
    pub ok: MSPRes,
    pub data: Vec<u8>,
}

impl MSPDev {
    pub fn new (defdev: &str) -> io::Result<MSPDev>   {
	match SerialPort::open(defdev, 115_200) {
            Ok(sd) => {
		let mut sd = sd;
                sd.set_read_timeout(Duration::from_millis(100))?;
		return Ok(MSPDev{sd,});
	    },
	    Err(_e) => return Err(_e),
	}
    }

    fn readmsg(&mut self) -> io::Result<MSPMsg> {
	let mut msg = MSPMsg::default();
	let mut n = States::Init;
	let mut crc = 0u8;
	let mut count = 0u16;
	let mut dirnok = false;
	let mut inp = [0u8; 256];
	let mut toc = 0;
	loop  {
            match self.sd.read(&mut inp) {
		Ok(0) => {
                    msg.cmd = 0;
                    msg.len = 0;
                    msg.ok = MSPRes::Fail;
                    return Ok(msg);
		},
		Ok(nbytes) => {
		    toc = 0;
                    for e in inp.iter().take(nbytes) {
			match n {
                            States::Init => {
				if *e == b'$' {
                                    n = States::M;
                                    msg.ok = MSPRes::Fail;
                                    msg.len = 0;
                                    msg.cmd = 0;
                                    dirnok = false;
				}
                            }
                            States::M => {
				n = match *e {
                                    b'M' => States::Dirn,
                                    b'X' => States::XHeader2,
                                    _ => States::Init,
				}
                            }
                            States::Dirn => match *e {
				b'!' => {
                                    n = States::Len;
				}
				b'>' => {
                                    n = States::Len;
                                    dirnok = true;
				}
				_ => n = States::Init,
                            },
                            States::XHeader2 => match *e {
				b'!' => n = States::XFlags,
				b'>' => {
                                    n = States::XFlags;
                                    dirnok = true;
				}
				_ => n = States::Init,
                            },
                            States::XFlags => {
				crc = crc8_dvb_s2(0, *e);
				n = States::XId1;
                            }
                            States::XId1 => {
				crc = crc8_dvb_s2(crc, *e);
				msg.cmd = *e as u16;
				n = States::XId2;
                            }
                            States::XId2 => {
				crc = crc8_dvb_s2(crc, *e);
				msg.cmd |= (*e as u16) << 8;
				n = States::XLen1;
                            }
                            States::XLen1 => {
				crc = crc8_dvb_s2(crc, *e);
				msg.len = *e as u16;
				n = States::XLen2;
                            }
                            States::XLen2 => {
				crc = crc8_dvb_s2(crc, *e);
				msg.len |= (*e as u16) << 8;
				if msg.len > 0 {
                                    n = States::XData;
                                    count = 0;
                                    msg.data = vec![0; msg.len.into()];
				} else {
                                    n = States::XChecksum;
				}
                            }
                            States::XData => {
				crc = crc8_dvb_s2(crc, *e);
				msg.data[count as usize] = *e;
				count += 1;
				if count == msg.len {
                                    n = States::XChecksum;
				}
                            }
                            States::XChecksum => {
				if crc != *e {
                                    msg.ok = MSPRes::Crc
				} else {
                                    msg.ok = if dirnok { MSPRes::Ok } else { MSPRes::Dirn };
				}
				return Ok(msg)
                            }
                            States::Len => {
				msg.len = *e as u16;
				crc = *e;
				n = States::Cmd;
                            }
                            States::Cmd => {
				msg.cmd = *e as u16;
				crc ^= *e;
				if msg.len == 0 {
                                    n = States::Crc;
				} else {
                                    msg.data = vec![0; msg.len.into()];
                                    n = States::Data;
                                    count = 0;
				}
                            }
                            States::Data => {
				msg.data[count as usize] = *e;
				crc ^= *e;
				count += 1;
				if count == msg.len {
                                    n = States::Crc;
				}
                            }
                            States::Crc => {
				if crc != *e {
                                    msg.ok = MSPRes::Crc;
				} else {
                                    msg.ok = if dirnok { MSPRes::Ok } else { MSPRes::Dirn };
				}
				return Ok(msg)
                            }
			}
                    }
		},
		Err(e) => {
		    if e.kind() == std::io::ErrorKind::TimedOut {
			toc += 1;
			if toc == 50 {
			    return Err(e)
			}
			if toc % 10 == 0 {
			    eprintln!("Read timeout");
			}
			continue
		    } else {
			return Err(e)
		    }
		},
            }
	}
    }

    pub fn write_msp(&mut self, cmd: u16,  payload: &[u8]) -> io::Result<()> {
	return self.sd.write_all(&encode_msp2(cmd, payload))
    }

    pub fn send_msp(&mut self, cmd: u16,  payload: &[u8]) -> io::Result<MSPMsg> {
	self.sd.write_all(&encode_msp2(cmd, payload))?;
	return self.readmsg()
    }
}

fn encode_msp2(cmd: u16, payload: &[u8]) -> Vec<u8> {
    let paylen = payload.len();
    let mut v = vec![0; paylen + 9];
    v[0] = b'$';
    v[1] = b'X';
    v[2] = b'<';
    v[3] = 0;
    v[4] = (cmd & 0xff) as u8;
    v[5] = (cmd >> 8) as u8;
    v[6] = (paylen & 0xff) as u8;
    v[7] = (paylen >> 8) as u8;
    v[8..paylen + 8].copy_from_slice(payload);
    let mut crc: u8 = 0;
    for e in v.iter().take(paylen + 8).skip(3) {
        crc = crc8_dvb_s2(crc, *e);
    }
    v[paylen + 8] = crc;
    v
}

#[allow(dead_code)]
fn encode_msp(cmd: u16, payload: &[u8]) -> Vec<u8> {
    let paylen = payload.len();
    //    let mut v: Vec<u8> = Vec::new();
    let mut v = vec![0; paylen + 6];
    v[0] = b'$';
    v[1] = b'M';
    v[2] = b'<';
    v[3] = paylen as u8;
    v[4] = cmd as u8;
    v[5..paylen + 5].copy_from_slice(payload);
    let mut crc: u8 = v[3] ^ v[4];
    for c in payload {
        crc ^= c;
    }
    v[paylen + 5] = crc;
    v
}

fn crc8_dvb_s2(mut c: u8, a: u8) -> u8 {
    c ^= a;
    for _ in 0..8 {
        if (c & 0x80) != 0 {
            c = (c << 1) ^ 0xd5
        } else {
            c <<= 1
        }
    }
    c
}

pub fn describe(v: &MSPMsg) -> Option<u16> {
    let nxt: Option<u16> = match v.cmd {
	MSG_API_VERSION => {
	    println!("MSP    : {}.{}", v.data[1], v.data[2]);
	    Some(MSG_NAME)
	},
	MSG_NAME => {
	    println!("Name   : {}", &String::from_utf8_lossy(&v.data));
	    Some(MSG_FC_VARIANT)
	},
	MSG_FC_VARIANT => {
	    println!("F/W    : {}", &String::from_utf8_lossy(&v.data[0..4]));
	    Some(MSG_FC_VERSION)
	},

	MSG_FC_VERSION => {
	    println!("Version: {}.{}.{}", v.data[0], v.data[1], v.data[2]);
	    Some(MSG_BUILD_INFO)
	},
	MSG_BUILD_INFO => {
	    print!("Build  : ");
            if v.len > 19 {
                println!("{} {} ({})",
			 &String::from_utf8_lossy(&v.data[0..11]),
			 &String::from_utf8_lossy(&v.data[11..19]),
			 &String::from_utf8_lossy(&v.data[19..])
                );
            } else {
		println!("Unknown");
	    }
            Some(MSG_BOARD_INFO)
        },
	MSG_BOARD_INFO => {
	    print!("Board  : ");
	    let board = if v.len > 8 {
                String::from_utf8_lossy(&v.data[9..])
            } else {
                String::from_utf8_lossy(&v.data[0..4])
            };
            println!("{}", &board);
	    Some(MSG_ANALOG)
	},
	MSG_ANALOG => {
            let volts: f32 = v.data[0] as f32 / 10.0;
            let amps: f32 = u16::from_le_bytes(v.data[5..7].try_into().unwrap()) as f32 / 100.0;
            println!("Analog : {:.1} volts, {:2} amps", volts, amps);
	    None
	},
	_ => {
	    println!("Unknown: {:?}", v);
	    None
	},
    };
    return nxt;
}
