extern crate itertools;

use std::env;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::Path;

include!("src/parse.rs");

#[cfg(feature = "latest")]
fn manuf() -> Result<BufReader<io::Cursor<String>>, anyhow::Error> {
    const LATEST_MANUF_URL: &str =
        "https://code.wireshark.org/review/gitweb?p=wireshark.git;a=blob_plain;f=manuf;hb=HEAD";

    let text = reqwest::blocking::get(LATEST_MANUF_URL)?.text()?;

    Ok(BufReader::new(io::Cursor::new(text)))
}

#[cfg(not(feature = "latest"))]
fn manuf() -> Result<BufReader<File>, io::Error> {
    File::open("src/manuf").map(BufReader::new)
}

fn main() -> Result<(), anyhow::Error> {
    let manuf = manuf()?;
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("vendors.rs");
    let mut out = File::create(&dest_path)?;

    out.write_all(
        b"#[doc(hidden)]
#[allow(clippy::unreadable_literal, clippy::type_complexity)]
pub const VENDORS: &[((EtherAddr, u64), (&str, &str))] = &[\n",
    )?;

    for ((prefix, prefix_len), (name, desc)) in parse(manuf) {
        let prefix_str = itertools::join(prefix.iter().map(|n| format!("0x{:02x}", n)), ", ");
        let prefix_mask = ((1u64 << prefix_len) - 1) << (48 - prefix_len);

        out.write_all(
            format!(
                "\t(([{}], 0x{:x}), ({:?}, {:?})),\n",
                prefix_str, prefix_mask, name, desc
            )
            .as_bytes(),
        )?;
    }

    out.write_all(b"];\n")?;

    Ok(())
}
