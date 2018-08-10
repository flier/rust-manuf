extern crate itertools;

use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

include!("src/parse.rs");

fn main() {
    let f = File::open("src/manuf").unwrap();
    let r = BufReader::new(f);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("vendors.rs");
    let mut out = File::create(&dest_path).unwrap();

    out.write_all(
        b"#[doc(hidden)]
#[cfg_attr(feature = \"cargo-clippy\", allow(unreadable_literal, type_complexity))]
pub const VENDORS: &[((EtherAddr, u64), (&str, &str))] = &[\n",
    ).unwrap();

    for ((prefix, prefix_len), (name, desc)) in parse(r) {
        let prefix_str = itertools::join(prefix.into_iter().map(|n| format!("0x{:02x}", n)), ", ");
        let prefix_mask = ((1u64 << prefix_len) - 1) << (48 - prefix_len);

        out.write_all(
            format!(
                "\t(([{}], 0x{:x}), ({:?}, {:?})),\n",
                prefix_str, prefix_mask, name, desc
            ).as_bytes(),
        ).unwrap();
    }

    out.write_all(b"];\n").unwrap();
}
