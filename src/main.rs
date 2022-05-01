use clap::{arg, Command};
use fsio::file;

fn main() {
    let args = clap_args().get_matches();

    match args.subcommand() {
        Some(("push", push_args)) => {
            let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
            if push_args.is_present("focused") {
                let count = push_args.occurrences_of("focused");
                for _ in 0..count {
                    let focused = xcb::get_input_focus(&conn).get_reply().unwrap().focus();
                    push(&conn, screen_num, focused);
                }
            }
            if push_args.is_present("selected") {
                let count = push_args.occurrences_of("selected");
                for _ in 0..count {
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
                    push(&conn, screen_num, selected);
                }
            }
            if push_args.is_present("window_id") {
                let options: Vec<&str> = push_args.values_of("window_id").unwrap().collect();
                for option in options {
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
                    push(&conn, screen_num, window);
                }
            }
        }
        Some(("pop", pop_args)) => {
            let (conn, _) = xcb::Connection::connect(None).unwrap();
            if pop_args.is_present("all") {
                let windows = file::read_text_file("/tmp/srsp.tmp").unwrap();
                for window in windows.lines() {
                    let window = window.parse::<u32>().unwrap();
                    pop(&conn, window);
                }
            }
            if pop_args.is_present("last") {
                let count = pop_args.occurrences_of("last");
                for _ in 0..count {
                    let last = file::read_text_file("/tmp/srsp.tmp")
                        .unwrap()
                        .lines()
                        .last()
                        .unwrap()
                        .parse::<u32>()
                        .unwrap();
                    pop(&conn, last);
                }
            }
            if pop_args.is_present("window_id") {
                let options: Vec<&str> = pop_args.values_of("window_id").unwrap().collect();
                for option in options {
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

pub fn push(conn: &xcb::Connection, screen_num: i32, window: u32) {
    let translate = xcb::translate_coordinates(
        conn,
        window,
        conn.get_setup()
            .roots()
            .nth(screen_num as usize)
            .unwrap()
            .root(),
        0,
        0,
    )
    .get_reply()
    .unwrap(); //Translates relative position to absolute position
    let geometry = xcb::get_geometry(conn, window).get_reply().unwrap();
    let values = &vec![
        (xcb::CONFIG_WINDOW_X as u16, translate.dst_x() as u32),
        (xcb::CONFIG_WINDOW_Y as u16, translate.dst_y() as u32),
        (xcb::CONFIG_WINDOW_WIDTH as u16, geometry.width() as u32),
        (xcb::CONFIG_WINDOW_HEIGHT as u16, geometry.height() as u32),
    ];

    let geometry = format!(
        "{},{},{},{},{},{},{},{}",
        values[0].0,
        values[0].1,
        values[1].0,
        values[1].1,
        values[2].0,
        values[2].1,
        values[3].0,
        values[3].1
    );

    file::ensure_exists("/tmp/srsp.tmp").unwrap();
    file::ensure_exists("/tmp/srsp-g.tmp").unwrap();
    file::append_text_file("/tmp/srsp.tmp", &format!("{}\n", window)).unwrap();
    file::append_text_file("/tmp/srsp-g.tmp", &format!("{:?}\n", geometry)).unwrap();
    xcb::unmap_window_checked(conn, window)
        .request_check()
        .unwrap();
    conn.flush();
}

pub fn pop(conn: &xcb::Connection, window: u32) {
    let mut new = String::new();
    let mut n = 1;
    for line in file::read_text_file("/tmp/srsp.tmp").unwrap().lines() {
        if line.parse::<u32>().unwrap() == window {
            continue;
        }
        n += 1;
        new.push_str(&format!("{}\n", line));
    }

    let mut new_g = String::new();

    let mut i = 1;

    for line in file::read_text_file("/tmp/srsp-g.tmp").unwrap().lines() {
        if i == n {
            continue;
        }
        i += 1;
        new_g.push_str(&format!("{}\n", line));
    }

    let value: String = file::read_text_file("/tmp/srsp-g.tmp")
        .unwrap()
        .lines()
        .take(n)
        .last()
        .unwrap()
        .to_string();

    let values: Vec<&str> = value
        .strip_prefix('\"')
        .unwrap()
        .strip_suffix('\"')
        .unwrap()
        .split(',')
        .map(|x| x.trim())
        .collect();

    let values = vec![
        (
            values[0].parse::<u16>().unwrap(),
            values[1].parse::<u32>().unwrap(),
        ),
        (
            values[2].parse::<u16>().unwrap(),
            values[3].parse::<u32>().unwrap(),
        ),
        (
            values[4].parse::<u16>().unwrap(),
            values[5].parse::<u32>().unwrap(),
        ),
        (
            values[6].parse::<u16>().unwrap(),
            values[7].parse::<u32>().unwrap(),
        ),
    ];

    file::write_text_file("/tmp/srsp.tmp", &new).unwrap();
    file::write_text_file("/tmp/srsp-g.tmp", &new_g).unwrap();
    xcb::map_window_checked(conn, window)
        .request_check()
        .unwrap();
    xcb::configure_window_checked(conn, window, &values)
        .request_check()
        .unwrap();
    conn.flush();
}
