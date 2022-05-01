use clap::{arg, Command};
use fsio::file;

fn main() {
    let args = clap_args().get_matches();

    let (conn, _) = xcb::Connection::connect(None).unwrap();
    if args.is_present("push") {
        let option = args.value_of("push").unwrap();
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
        push(window);
        xcb::unmap_window_checked(&conn, window)
            .request_check()
            .unwrap();
        conn.flush();
    } else if args.is_present("pop") {
        let option = args.value_of("pop").unwrap();
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
        pop(&window);
        xcb::map_window_checked(&conn, window)
            .request_check()
            .unwrap();
        conn.flush();
    }
}

pub fn clap_args() -> Command<'static> {
    let app = Command::new("srsp")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            arg!(-i - -push)
                .required(false)
                .takes_value(true)
                .help("Push the focused window into the scratchpad"),
        )
        .arg(
            arg!(-o - -pop)
                .required(false)
                .takes_value(true)
                .help("Pop out a window from the scratchpad"),
        );
    app
}

pub fn push(window: u32) {
    file::ensure_exists("/tmp/srsp.tmp").unwrap();
    file::append_text_file("/tmp/srsp.tmp", &format!("{}\n", window)).unwrap();
}

pub fn pop(window: &u32) {
    let mut new = String::new();
    for line in file::read_text_file("/tmp/srsp.tmp").unwrap().lines() {
        if line.parse::<u32>().unwrap() == *window {
            continue;
        }
        new.push_str(&format!("{}\n", line));
    }
    file::write_text_file("/tmp/srsp.tmp", &new).unwrap();
}
