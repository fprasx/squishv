use std::collections::HashMap;

use anyhow::{bail, ensure, Context};

use crate::parse::{LoadOp, StoreOp};

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Config {
    /// The default value for unitialized memory locations. `None` means there is
    /// not default value and it is an error to access an unitialized location.
    /// Defaults to `None`.
    default_value: Option<u8>,

    /// Whether or not to allow unaligned accesses. Defaults to `false`.
    allow_unaligned: bool,
}

impl Config {
    pub fn new(default_value: Option<u8>, allow_unaligned: bool) -> Self {
        Self {
            default_value,
            allow_unaligned,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
/// Byte addressible memory that handles unitialized values and unaligned access.
///
/// See [`Config`] for more details.
pub struct Memory {
    config: Config,

    // Bytes are stored in little-endian order
    mem: HashMap<i32, u8>,
}

impl Memory {
    pub fn load(&self, addr: i32, op: LoadOp) -> anyhow::Result<i32> {
        match op {
            LoadOp::Lw => {
                if !self.config.allow_unaligned {
                    ensure!(addr & 0b11 == 0, "uanligned access: address = {addr:#010x}");
                }
                let data = self
                    .load_bytes::<4>(addr)
                    .with_context(|| format!("failed to load word at {addr:#010x}"))?;
                Ok(i32::from_le_bytes(data))
            }
            LoadOp::Lh => {
                if !self.config.allow_unaligned {
                    ensure!(addr & 0b1 == 0, "unaligned access: address = {addr:#010x}");
                }
                let data = self
                    .load_bytes::<2>(addr)
                    .with_context(|| format!("failed to load half word at {addr:#010x}"))?;
                // Sign extends
                Ok(i16::from_le_bytes(data) as i32)
            }
            LoadOp::Lhu => {
                if !self.config.allow_unaligned {
                    ensure!(addr & 0b1 == 0, "uanligned access: address = {addr:#010x}");
                }
                let data = self
                    .load_bytes::<2>(addr)
                    .with_context(|| format!("failed to load half word at {addr:#010x}"))?;
                // First cast to u32 to zero extend, then cast to i32
                Ok((u16::from_le_bytes(data) as u32) as i32)
            }
            LoadOp::Lb => {
                let data = self
                    .load_bytes::<1>(addr)
                    .with_context(|| format!("failed to load byte at {addr:#010x}"))?;
                // Sign extends
                Ok(i8::from_le_bytes(data) as i32)
            }
            LoadOp::Lbu => {
                let data = self
                    .load_bytes::<1>(addr)
                    .with_context(|| format!("failed to load byte at {addr:#010x}"))?;
                // First cast to u32 to zero extend, then cast to i32
                Ok((u8::from_le_bytes(data) as u32) as i32)
            }
        }
    }

    /// Load `N` bytes, starting at the base address. Returns an error if any of
    /// then is unitialized.
    fn load_bytes<const N: usize>(&self, base_addr: i32) -> anyhow::Result<[u8; N]> {
        let mut data = [0u8; N];
        for (offset, spot) in data.iter_mut().enumerate() {
            let addr = base_addr + (offset as i32);
            let Some(byte) = self.mem.get(&addr).copied().or(self.config.default_value) else {
                bail!("access to unitialized address: {addr:#010x}")
            };
            *spot = byte;
        }
        Ok(data)
    }

    /// Store `N` bytes, starting at the base address
    fn store_bytes<const N: usize>(&mut self, base_addr: i32, bytes: [u8; N]) {
        for (offset, byte) in bytes.iter().enumerate() {
            let addr = base_addr + (offset as i32);
            self.mem.insert(addr, *byte);
        }
    }

    /// Store a value at a certain address, returning the value that was previously
    /// there, if any.
    pub fn store(&mut self, addr: i32, val: i32, op: StoreOp) -> anyhow::Result<()> {
        // Note: casting to a smaller integer type truncates, which is what we want
        match op {
            StoreOp::Sw => {
                if !self.config.allow_unaligned {
                    ensure!(addr & 0b11 == 0, "uanligned access: address = {addr:#010x}");
                }
                self.store_bytes(addr, val.to_le_bytes());
            }
            StoreOp::Sh => {
                if !self.config.allow_unaligned {
                    ensure!(addr & 0b1 == 0, "uanligned access: address = {addr:#010x}");
                }
                self.store_bytes(addr, (val as i16).to_le_bytes());
            }
            StoreOp::Sb => {
                self.store_bytes(addr, (val as i8).to_le_bytes());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        map,
        parse::{LoadOp, StoreOp},
    };

    use super::*;

    #[test]
    fn loads() {
        let mut mem: Memory = Default::default();
        assert_eq!(
            Memory {
                config: Config {
                    default_value: None,
                    allow_unaligned: false
                },
                mem: map![]
            },
            mem
        );
        assert!(mem.load(0x0, LoadOp::Lw).is_err());
        assert!(mem.load(0x0, LoadOp::Lh).is_err());
        assert!(mem.load(0x0, LoadOp::Lb).is_err());
        assert!(mem.load(0x0, LoadOp::Lhu).is_err());
        assert!(mem.load(0x0, LoadOp::Lbu).is_err());

        mem.store(0x40, 0x1234abcd, StoreOp::Sw).unwrap();

        assert_eq!(mem.load(0x40, LoadOp::Lw).unwrap(), 0x1234abcd);
        // out of bounds
        assert!(mem.load(0x41, LoadOp::Lw).is_err());

        assert_eq!(mem.load(0x40, LoadOp::Lh).unwrap(), 0xffffabcd_u32 as i32);
        assert!(mem.load(0x41, LoadOp::Lh).is_err());
        assert_eq!(mem.load(0x42, LoadOp::Lh).unwrap(), 0x00001234_u32 as i32);
        // out of bounds
        assert!(mem.load(0x43, LoadOp::Lh).is_err());

        assert_eq!(mem.load(0x40, LoadOp::Lb).unwrap(), 0xffffffcd_u32 as i32);
        assert_eq!(mem.load(0x41, LoadOp::Lb).unwrap(), 0xffffffab_u32 as i32);
        assert_eq!(mem.load(0x42, LoadOp::Lb).unwrap(), 0x00000034_u32 as i32);
        assert_eq!(mem.load(0x43, LoadOp::Lb).unwrap(), 0x00000012_u32 as i32);
        // out of bounds
        assert!(mem.load(0x44, LoadOp::Lb).is_err());

        assert_eq!(mem.load(0x40, LoadOp::Lhu).unwrap(), 0x0000abcd);
        assert!(mem.load(0x41, LoadOp::Lhu).is_err());
        assert_eq!(mem.load(0x42, LoadOp::Lhu).unwrap(), 0x00001234);
        // out of bounds
        assert!(mem.load(0x43, LoadOp::Lhu).is_err());

        assert_eq!(mem.load(0x40, LoadOp::Lbu).unwrap(), 0x000000cd);
        assert_eq!(mem.load(0x41, LoadOp::Lbu).unwrap(), 0x000000ab);
        assert_eq!(mem.load(0x42, LoadOp::Lbu).unwrap(), 0x00000034);
        assert_eq!(mem.load(0x43, LoadOp::Lbu).unwrap(), 0x00000012);
        // out of bounds
        assert!(mem.load(0x44, LoadOp::Lbu).is_err());

        // Unaligned access
        mem.config.allow_unaligned = true;
        mem.store(0x44, 0x1234abcd, StoreOp::Sw).unwrap();
        assert_eq!(mem.load(0x41, LoadOp::Lw).unwrap(), 0xcd1234ab_u32 as i32);
        assert_eq!(mem.load(0x41, LoadOp::Lh).unwrap(), 0x000034ab_u32 as i32);
        assert_eq!(mem.load(0x41, LoadOp::Lhu).unwrap(), 0x000034ab);

        // Default values
        mem.config.default_value = Some(0xaa);
        assert_eq!(mem.load(0x48, LoadOp::Lw).unwrap(), 0xaaaaaaaa_u32 as i32);
        assert_eq!(mem.load(0x48, LoadOp::Lh).unwrap(), 0xffffaaaa_u32 as i32);
        assert_eq!(mem.load(0x48, LoadOp::Lhu).unwrap(), 0xaaaa);
        assert_eq!(mem.load(0x48, LoadOp::Lb).unwrap(), 0xffffffaa_u32 as i32);
        assert_eq!(mem.load(0x48, LoadOp::Lbu).unwrap(), 0xaa);
    }

    #[test]
    fn stores() {
        let mut mem: Memory = Default::default();

        assert_eq!(mem.mem, map![]);

        mem.store(0x20, 0x1234abcd, StoreOp::Sw).unwrap();
        assert_eq!(
            mem.mem,
            map![
                0x20 => 0xcd,
                0x21 => 0xab,
                0x22 => 0x34,
                0x23 => 0x12,
            ]
        );
        // Truncates
        mem.store(0x22, 0x1234abcd, StoreOp::Sh).unwrap();
        assert_eq!(
            mem.mem,
            map![
                0x20 => 0xcd,
                0x21 => 0xab,
                0x22 => 0xcd,
                0x23 => 0xab,
            ]
        );
        mem.store(0x23, 0x1234abcd, StoreOp::Sb).unwrap();
        assert_eq!(
            mem.mem,
            map![
                0x20 => 0xcd,
                0x21 => 0xab,
                0x22 => 0xcd,
                0x23 => 0xcd,
            ]
        );

        // Unaligned
        assert!(mem.store(0x21, 0x1234abcd, StoreOp::Sw).is_err());
        assert!(mem.store(0x21, 0x1234abcd, StoreOp::Sh).is_err());
        mem.config.allow_unaligned = true;
        mem.store(0x21, 0x1234abcd, StoreOp::Sw).unwrap();
        assert_eq!(
            mem.mem,
            map![
                0x20 => 0xcd,
                0x21 => 0xcd,
                0x22 => 0xab,
                0x23 => 0x34,
                0x24 => 0x12,

            ]
        );
        mem.store(0x23, 0x1234abcd, StoreOp::Sh).unwrap();
        assert_eq!(
            mem.mem,
            map![
                0x20 => 0xcd,
                0x21 => 0xcd,
                0x22 => 0xab,
                0x23 => 0xcd,
                0x24 => 0xab,
            ]
        );
    }
}
