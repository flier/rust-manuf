use std::io::BufRead;
use std::iter::repeat;

/// The number of bytes in an ethernet (MAC) address.
pub const ETHER_ADDR_LEN: usize = 6;

/// Structure of a 48-bit Ethernet address.
pub type EtherAddr = [u8; ETHER_ADDR_LEN];

/// Parse `manuf` file for vendor's `((prefix, prefix_length), (name, description))`.
///
/// ## Example
///
/// ```rust,no_run
/// extern crate manuf;
///
/// use std::fs::File;
/// use std::io::BufReader;
///
/// fn main() {
///     let f = File::open("manuf").unwrap();
///     let r = BufReader::new(f);
///
///     for ((prefix, prefix_len), (name, desc)) in manuf::parse(r) {
///         println!("{:?}/{}\t{}\t{}", prefix, prefix_len, name, desc)
///     }
/// }
/// ```
///
/// **Notes:** `manuf` file was generated by [the Wireshark project](https://github.com/wireshark/wireshark/blob/master/manuf).
pub fn parse<R: BufRead>(r: R) -> impl Iterator<Item = ((EtherAddr, u32), (String, String))> {
    r.lines().flat_map(|line| line).flat_map(|line| {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            None
        } else {
            let (prefix_str, name, desc) = {
                let mut v = line.splitn(3, '\t').chain(repeat(""));

                (v.next()?, v.next()?, v.next()?)
            };

            let (prefix_str, prefix_len) = {
                let mut v = prefix_str.split('/');
                (
                    v.next()?,
                    v.next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_else(|| 8 * prefix_str.split(':').count()),
                )
            };

            let prefix = {
                let mut v = prefix_str
                    .split(':')
                    .flat_map(|s| u8::from_str_radix(s, 16))
                    .chain(repeat(0));
                [
                    v.next()?,
                    v.next()?,
                    v.next()?,
                    v.next()?,
                    v.next()?,
                    v.next()?,
                ]
            };

            Some((
                (prefix, prefix_len as u32),
                (name.to_owned(), desc.to_owned()),
            ))
        }
    })
}
