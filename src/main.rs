use clap::{arg, Command};
use fsio::file;

fn main() {
    let args = clap_args().get_matches();

    match args.subcommand() {
        Some(("push", push_args)) => {
            if push_args.is_present("focused") {
                let count = push_args.occurrences_of("focused");
                for _ in 0..count {
                    let (conn, _) = xcb::Connection::connect(None).unwrap();
                    let focused = xcb::get_input_focus(&conn).get_reply().unwrap().focus();
                    push(conn, focused);
                }
            }
            if push_args.is_present("selected") {
                let count = push_args.occurrences_of("selected");
                for _ in 0..count {
                    let (conn, _) = xcb::Connection::connect(None).unwrap();
                    let selected = std::process::Command::new("sh")
                        .arg("-c")
                        .arg("xdotool selectwindow")
                        .output()
                        .unwrap()
                        .stdout;
                    let selected = String::from_utf8(selected)
                        .unwrap()
                        .trim()
                        .parse::<u32>()
                        .unwrap();
                    push(conn, selected);
                }
            }
            if push_args.is_present("window_id") {
                let options: Vec<&str> = push_args.values_of("window_id").unwrap().collect();
                for option in options {
                    let (conn, _) = xcb::Connection::connect(None).unwrap();
                    let window = match option {
                        "focused" => xcb::get_input_focus(&conn).get_reply().unwrap().focus(),
                        "selected" => {
                            let selected = std::process::Command::new("sh")
                                .arg("-c")
                                .arg("xdotool selectwindow")
                                .output()
                                .unwrap()
                                .stdout;
                            String::from_utf8(selected)
                                .unwrap()
                                .trim()
                                .parse::<u32>()
                                .unwrap()
                        }
                        _ => option.parse::<u32>().unwrap(),
                    };
                    push(conn, window);
                }
            }
        }
        Some(("pop", pop_args)) => {
            if pop_args.is_present("all") {
                let windows = file::read_text_file("/tmp/srsp.tmp").unwrap();
                for window in windows.lines() {
                    let (conn, _) = xcb::Connection::connect(None).unwrap();
                    let window = window.parse::<u32>().unwrap();
                    pop(conn, window);
                }
            }
            if pop_args.is_present("last") {
                let count = pop_args.occurrences_of("last");
                for _ in 0..count {
                    let (conn, _) = xcb::Connection::connect(None).unwrap();
                    let last = file::read_text_file("/tmp/srsp.tmp")
                        .unwrap()
                        .lines()
                        .last()
                        .unwrap()
                        .parse::<u32>()
                        .unwrap();
                    pop(conn, last);
                }
            }
            if pop_args.is_present("window_id") {
                let options: Vec<&str> = pop_args.values_of("window_id").unwrap().collect();
                for option in options {
                    let (conn, _) = xcb::Connection::connect(None).unwrap();
                    let window = match option {
                        "last" => file::read_text_file("/tmp/srsp.tmp")
                            .unwrap()
                            .lines()
                            .last()
                            .unwrap()
                            .parse::<u32>()
                            .unwrap(),
                        _ => option.parse::<u32>().unwrap(),
                    };
                    xcb::map_window_checked(&conn, window)
                        .request_check()
                        .unwrap();
                    conn.flush();
                }
                file::write_text_file("/tmp/srsp.tmp", "").unwrap();
            }
        }
        _ => unreachable!(),
    }
}

pub fn clap_args() -> Command<'static> {
    let app = Command::new("srsp")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("push")
                .about("Push the windows into the scratchpad")
                .arg(
                    arg!(-i - -window_id)
                        .required(false)
                        .takes_value(true)
                        .multiple_values(true)
                        .help("Push the window into the scratchpad"),
                )
                .arg(
                    arg!(-f - -focused)
                        .required(false)
                        .takes_value(false)
                        .multiple_occurrences(true)
                        .help("Push the focused window into the scratchpad"),
                )
                .arg(
                    arg!(-s - -selected)
                        .required(false)
                        .takes_value(false)
                        .multiple_occurrences(true)
                        .help("Push the selected window into the scratchpad"),
                ),
        )
        .subcommand(
            Command::new("pop")
                .about("Pop out windows from the scratchpad")
                .arg(
                    arg!(-i - -window_id)
                        .required(false)
                        .takes_value(true)
                        .multiple_values(true)
                        .help("Pop out the window from the scratchpad"),
                )
                .arg(
                    arg!(-l - -last)
                        .required(false)
                        .takes_value(false)
                        .multiple_occurrences(true)
                        .help("Pop out the last window from the scratchpad"),
                )
                .arg(
                    arg!(-a - -all)
                        .required(false)
                        .takes_value(false)
                        .conflicts_with_all(&["window_id", "last"])
                        .help("Pop out the last window from the scratchpad"),
                ),
        );
    app
}

pub fn push(conn: xcb::Connection, window: u32) {
    file::ensure_exists("/tmp/srsp.tmp").unwrap();
    file::append_text_file("/tmp/srsp.tmp", &format!("{}\n", window)).unwrap();
    xcb::unmap_window_checked(&conn, window)
        .request_check()
        .unwrap();
    conn.flush();
}

pub fn pop(conn: xcb::Connection, window: u32) {
    let mut new = String::new();
    for line in file::read_text_file("/tmp/srsp.tmp").unwrap().lines() {
        if line.parse::<u32>().unwrap() == window {
            continue;
        }
        new.push_str(&format!("{}\n", line));
    }
    file::write_text_file("/tmp/srsp.tmp", &new).unwrap();
    xcb::map_window_checked(&conn, window)
        .request_check()
        .unwrap();
    conn.flush();
}
