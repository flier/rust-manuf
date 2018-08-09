extern crate itertools;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::iter::repeat;
use std::path::Path;

fn main() {
    let f = File::open("src/manuf").unwrap();
    let r = BufReader::new(f);

    let lines = r.lines().flat_map(|line| line).flat_map(|line| {
        let line = line.trim();

        if line.is_empty() || line.starts_with("#") {
            None
        } else {
            let v = line.splitn(3, '\t').collect::<Vec<_>>();

            if v.len() < 3 {
                None
            } else {
                let prefix = v[0].trim();
                let vendor = v[1].trim();
                let desc = v[2].trim();

                let v = prefix.split('/').collect::<Vec<_>>();
                let (prefix, prefix_len) = if v.len() == 2 {
                    (v[0], v[1].parse().unwrap())
                } else {
                    (prefix, 8 * prefix.split(':').count())
                };
                let prefix = itertools::join(
                    prefix
                        .split(':')
                        .map(|s| usize::from_str_radix(s, 16).unwrap())
                        .chain(repeat(0))
                        .take(6)
                        .map(|n| format!("0x{:02x}", n)),
                    ", ",
                );
                let prefix_mask = ((1u64 << prefix_len) - 1) << (48 - prefix_len);

                Some(format!(
                    "\t((&[{}], 0x{:x}), ({:?}, {:?})),\n",
                    prefix, prefix_mask, vendor, desc
                ))
            }
        }
    });

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("vendors.rs");
    let mut out = File::create(&dest_path).unwrap();

    out.write_all(
        b"#[doc(hidden)]\npub const VENDORS: &[((&EtherAddr, u64), (&str, &str))] = &[\n",
    ).unwrap();
    for line in lines {
        out.write_all(line.as_bytes()).unwrap();
    }
    out.write_all(b"];\n").unwrap();
}
