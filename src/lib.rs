//! `rust-manuf` is a rust library provides Ethernet vendor codes, and well-known MAC addresses.
//!
//! ## Example
//!
//! ```rust
//! #[macro_use]
//! extern crate manuf;
//!
//! fn main() {
//!     assert_eq!(
//!         manuf::vendor(&[0x8c, 0x85, 0x90, 0x0b, 0xcb, 0x9e]),
//!         Some(("Apple", "Apple, Inc."))
//!     );
//!
//!     assert!(
//!         manuf::prefix("Apple")
//!             .any(|prefix| prefix == (&[0x8c, 0x85, 0x90, 0x00, 0x00, 0x00], 24))
//!     );
//! }
//! ```
extern crate byteorder;

mod parse;

pub use parse::{parse, EtherAddr, ETHER_ADDR_LEN};

use byteorder::{ByteOrder, NetworkEndian};

include!(concat!(env!("OUT_DIR"), "/vendors.rs"));

/// Find vendor name and description base on an ethernet (MAC) address.
///
/// ## Example
///
/// ```rust
/// assert_eq!(
///     manuf::vendor(&[0x8c, 0x85, 0x90, 0x0b, 0xcb, 0x9e]),
///     Some(("Apple", "Apple, Inc."))
/// );
/// ```
pub fn vendor(addr: &EtherAddr) -> Option<(&'static str, &'static str)> {
    match VENDORS.binary_search_by_key(addr, |&((prefix, _), _)| *prefix) {
        Ok(idx) => Some(VENDORS[idx].1),
        Err(idx) if idx > 0 => {
            let ((prefix, mask), vendor) = VENDORS[idx - 1];

            if NetworkEndian::read_uint(addr, ETHER_ADDR_LEN) & mask
                == NetworkEndian::read_uint(prefix, ETHER_ADDR_LEN)
            {
                Some(vendor)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Find vendor's prefix and mask for the ethernet (MAC) address.
///
/// ## Example
///
/// ```rust
/// assert!(
///     manuf::prefix("Apple")
///         .any(|prefix| prefix == (&[0x8c, 0x85, 0x90, 0x00, 0x00, 0x00], 24))
/// );
/// ```
pub fn prefix<S: AsRef<str>>(s: S) -> impl Iterator<Item = (&'static EtherAddr, u32)> {
    VENDORS
        .iter()
        .filter(move |(_, (name, _))| *name == s.as_ref())
        .map(|&((prefix, prefix_max), _)| (prefix, prefix_max.count_ones()))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn test_vendor() {
        assert_eq!(VENDORS.len(), 34875);
        assert_eq!(
            vendor(&[0x8c, 0x85, 0x90, 0x0b, 0xcb, 0x9e]),
            Some(("Apple", "Apple, Inc."))
        );
        assert_eq!(
            vendor(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            Some((
                "00:00:00",
                "Officially Xerox, but 0:0:0:0:0:0 is more common"
            ))
        );
        assert_eq!(
            vendor(&[0xFC, 0xFF, 0xAA, 0x11, 0x22, 0x33]),
            Some(("IeeeRegi", "IEEE Registration Authority"))
        );
        assert_eq!(
            vendor(&[0x50, 0x50, 0x2a, 0x00, 0x00, 0x00]),
            Some(("Egardia", ""))
        );
        assert_eq!(vendor(&[0x0a, 0x00, 0x27, 0x00, 0x00, 0x00]), None);

        for &((prefix, prefix_mask), (name, desc)) in VENDORS {
            if name == "IeeeRegi" {
                continue;
            }

            assert_eq!(vendor(prefix), Some((name, desc)));

            let mut addr = [0; ETHER_ADDR_LEN];

            NetworkEndian::write_uint(
                &mut addr,
                NetworkEndian::read_uint(prefix, ETHER_ADDR_LEN)
                    + u64::from(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .subsec_nanos(),
                    )
                        % (1 << (ETHER_ADDR_LEN * 8 - prefix_mask.count_ones() as usize)),
                ETHER_ADDR_LEN,
            );

            assert_eq!(vendor(&addr), Some((name, desc)));
        }
    }

    #[test]
    fn test_prefix() {
        assert!(
            prefix("Apple").any(|prefix| prefix == (&[0x8c, 0x85, 0x90, 0x00, 0x00, 0x00], 24))
        );
        assert_eq!(
            prefix("00:00:00").collect::<Vec<_>>(),
            vec![(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 24)]
        );
        assert!(
            prefix("IeeeRegi").any(|prefix| prefix == (&[0xFC, 0xFF, 0xAA, 0x00, 0x00, 0x00], 24))
        );

        for &((data, mask), (name, _)) in VENDORS {
            assert!(prefix(name).any(|prefix| prefix == (data, mask.count_ones())));
        }
    }
}
