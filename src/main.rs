use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::{Shutdown, TcpStream},
    path::PathBuf,
    thread,
};

use rustyline::{error::ReadlineError, history::History};
use serde_json::Value;

#[derive(Default, Clone)]
struct Config {
    json: bool,
}

fn main() {
    let mut args = env::args().skip(1);
    let host = args.next();
    let port = args.next().and_then(|p| p.parse().ok());

    let mut conf = Config::default();
    for arg in args {
        match arg.as_str() {
            "--json" | "-j" => conf.json = true,
            _ => {}
        }
    }

    let (Some(host), Some(port)) = (host, port) else {
        show_help();
        return;
    };

    let Ok(mut conn) = TcpStream::connect((host.as_ref(), port)) else {
        println!("Couldn't connect");
        return;
    };

    let Ok(rd) = conn.try_clone() else {
        println!("Couldn't split connection");
        return;
    };

    let read_loop = thread::spawn(move || {
        if conf.json {
            let src = serde_json::Deserializer::from_reader(rd);
            for value in src.into_iter::<Value>() {
                match value {
                    Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap()),
                    Err(e) => println!("{e}"),
                }
            }
        } else {
            let mut src = BufReader::new(rd);
            let mut buffer = String::new();
            while let Ok(n) = src.read_line(&mut buffer) {
                if n == 0 {
                    break;
                }
                print!("{}", buffer);
                buffer.clear();
            }
        }
    });

    let Ok(mut repl) = rustyline::DefaultEditor::new() else {
        println!("Couldn't create repl");
        return;
    };

    let history_file = init_history_file(&host, port);

    if let Some(history) = &history_file {
        _ = repl.history_mut().load(history);
    }

    loop {
        match repl.readline("") {
            Ok(line) => {
                _ = repl.add_history_entry(line.as_str());
                conn.write(&line.into_bytes()).unwrap();
                conn.write(b"\n").unwrap();
            }
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                _ = conn.shutdown(Shutdown::Both);
                break;
            }
            Err(e) => {
                println!("Error reading line: {e}");
                conn.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
    }

    if let Some(history) = history_file {
        _ = repl.history_mut().save(&history);
    }
    _ = read_loop.join();
}

fn init_history_file(host: &str, port: u16) -> Option<PathBuf> {
    let history_dir = dirs::cache_dir().map(|cache| cache.join("replance"))?;
    fs::create_dir_all(&history_dir).ok()?;
    Some(history_dir.join(format!("{host}:{port}")))
}

fn show_help() {
    println!("Replance - REPL for nc");
    println!("Usage - rnc <HOST> <PORT>");
}
