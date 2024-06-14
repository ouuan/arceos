#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::io::{self, prelude::*};
use std::net::{TcpStream, ToSocketAddrs};

#[cfg(feature = "dns")]
const DEST: &str = "ifconfig.me:80";
#[cfg(not(feature = "dns"))]
const DEST: &str = "34.117.118.44:80";

const REQUEST: &str = "\
GET / HTTP/1.1\r\n\
Host: ifconfig.me\r\n\
Accept: */*\r\n\
\r\n";

fn client() -> io::Result<()> {
    for addr in DEST.to_socket_addrs()? {
        println!("dest: {} ({})", DEST, addr);
    }

    let mut stream = TcpStream::connect(DEST)?;
    stream.write_all(REQUEST.as_bytes())?;
    let mut buf = [0; 2048];
    let n = stream.read(&mut buf)?;
    let response = core::str::from_utf8(&buf[..n]).unwrap();
    println!("{}", response); // longer response need to handle tcp package problems.
    Ok(())
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Hello, simple http client!");
    client().expect("test http client failed");
}
