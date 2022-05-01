use clap::{arg, Command};
use fsio::file;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let args = clap_args().get_matches();

    let (conn, _) = xcb::Connection::connect(None).unwrap();
    if args.is_present("in") {
        let focused = xcb::get_input_focus(&conn).get_reply().unwrap().focus();
        push(focused);
        xcb::unmap_window_checked(&conn, focused)
            .request_check()
            .unwrap();
        conn.flush();
    } else if args.is_present("out") {
        let out = pop();
        xcb::map_window_checked(&conn, out).request_check().unwrap();
        conn.flush();
    }
}

pub fn clap_args() -> Command<'static> {
    let app = Command::new("srsp")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            arg!(-i --in)
                .required(false)
                .takes_value(false)
                .help("Push the focused window into the scratchpad"),
        )
        .arg(
            arg!(-o - -out)
                .required(false)
                .takes_value(false)
                .help("Pop out the last window from the scratchpad"),
        );
    app
}

pub fn push(window: u32) {
    file::ensure_exists("/tmp/srsp.tmp").unwrap();
    file::append_text_file("/tmp/srsp.tmp", &format!("{}\n", window)).unwrap();
}

pub fn pop() -> u32 {
    let content = file::read_text_file("/tmp/srsp.tmp").unwrap();
    let last_window = content.lines().last().unwrap();
    let new_content = content
        .lines()
        .take(content.lines().count() - 1)
        .map(|i| format!("{}\n", i))
        .collect::<String>();
    file::write_text_file("/tmp/srsp.tmp", &new_content).unwrap();
    last_window.parse::<u32>().unwrap()
}
