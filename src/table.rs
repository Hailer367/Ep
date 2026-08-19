use std::fs::File;
use std::io::{self, Error, ErrorKind, Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ec;

const MAGIC: &[u8; 8] = b"EPHILTBL";
const HEADER_LEN: usize = 48;

pub struct Table {
    slots: u64,
    count: u64,
    data: Vec<AtomicU64>,
}

fn slot_index(h160: &[u8; 20], slots: u64) -> u64 {
    let v = u64::from_le_bytes([
        h160[0], h160[1], h160[2], h160[3], h160[4], h160[5], h160[6], h160[7],
    ]);
    v & (slots - 1)
}

impl Table {
    pub fn new(count: u64, load: f64) -> Table {
        let mut slots = ((count as f64) * load).ceil() as u64;
        if slots == 0 {
            slots = 1;
        }
        slots = slots.next_power_of_two();
        if slots < 1 {
            slots = 1;
        }
        let mut data = Vec::with_capacity(slots as usize);
        for _ in 0..slots {
            data.push(AtomicU64::new(0));
        }
        Table {
            slots,
            count,
            data,
        }
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn slots(&self) -> u64 {
        self.slots
    }

    pub fn insert(&self, n: u64, h160: &[u8; 20]) {
        let mut idx = slot_index(h160, self.slots);
        for _ in 0..self.slots {
            let slot = &self.data[idx as usize];
            if slot
                .compare_exchange_weak(0, n, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }
            idx = (idx + 1) % self.slots;
        }
        panic!("table full (increase --load)");
    }

    pub fn lookup(&self, h160: &[u8; 20]) -> Option<u64> {
        let mut idx = slot_index(h160, self.slots);
        for _ in 0..self.slots {
            let val = self.data[idx as usize].load(Ordering::Relaxed);
            if val == 0 {
                return None;
            }
            if ec::hash160_of_n(val) == *h160 {
                return Some(val);
            }
            idx = (idx + 1) % self.slots;
        }
        None
    }

    fn disk_width(&self) -> u32 {
        if self.count <= u32::MAX as u64 {
            4
        } else {
            8
        }
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        let width = self.disk_width();

        file.write_all(MAGIC)?;
        file.write_all(&1u32.to_le_bytes())?; // version
        file.write_all(&width.to_le_bytes())?;
        file.write_all(&self.count.to_le_bytes())?;
        file.write_all(&self.slots.to_le_bytes())?;
        file.write_all(&[0u8; 16])?; // reserved [32..48]

        if width == 4 {
            let mut tmp = [0u8; 4];
            for slot in &self.data {
                let v = slot.load(Ordering::Relaxed) as u32;
                tmp.copy_from_slice(&v.to_le_bytes());
                file.write_all(&tmp)?;
            }
        } else {
            let mut tmp = [0u8; 8];
            for slot in &self.data {
                let v = slot.load(Ordering::Relaxed);
                tmp.copy_from_slice(&v.to_le_bytes());
                file.write_all(&tmp)?;
            }
        }
        file.sync_all()?;
        Ok(())
    }

    pub fn load(path: &str) -> io::Result<Table> {
        let mut file = File::open(path)?;
        let mut header = [0u8; HEADER_LEN];
        file.read_exact(&mut header)?;
        if &header[0..8] != MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "bad magic"));
        }
        let width = u32::from_le_bytes(header[12..16].try_into().unwrap());
        let count = u64::from_le_bytes(header[16..24].try_into().unwrap());
        let slots = u64::from_le_bytes(header[24..32].try_into().unwrap());
        if width != 4 && width != 8 {
            return Err(Error::new(ErrorKind::InvalidData, "bad width"));
        }

        let mut data = Vec::with_capacity(slots as usize);
        if width == 4 {
            let mut tmp = [0u8; 4];
            for _ in 0..slots {
                file.read_exact(&mut tmp)?;
                let v = u32::from_le_bytes(tmp) as u64;
                data.push(AtomicU64::new(v));
            }
        } else {
            let mut tmp = [0u8; 8];
            for _ in 0..slots {
                file.read_exact(&mut tmp)?;
                let v = u64::from_le_bytes(tmp);
                data.push(AtomicU64::new(v));
            }
        }

        Ok(Table {
            slots,
            count,
            data,
        })
    }
}