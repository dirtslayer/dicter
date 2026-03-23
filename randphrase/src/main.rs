use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::env;

fn get_dict_name() -> String {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--dict" {
            if let Some(val) = args.next() {
                return val;
            }
        }
    }
    "freedict-eng-fra".to_string()
}

fn get_socket_path(dict: &str) -> String {
    format!("/tmp/phrases-{}.sock", dict)
}

static PHRASES: Lazy<Vec<String>> = Lazy::new(|| {

let dict = get_dict_name();
let path = format!("/usr/share/dictd/{}.index", dict);

if !Path::new(&path).exists() {
    eprintln!("Dictionary '{}' not found at {}.", dict, path);
    eprintln!("Try: apt search freedict");
    std::process::exit(1);
}

    let file = File::open(&path).expect("cannot open dictionary file");
    let reader = BufReader::new(file);

    let mut phrases = Vec::new();

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            if let Some(phrase) = parse_line(&line) {
                phrases.push(phrase);
            }
        }
    }

    if phrases.is_empty() {
        eprintln!("Warning: no phrases loaded from {}", path);
    } else {
        eprintln!("Loaded {} phrases from {}", phrases.len(), path);
    }

    phrases
});

fn parse_line(line: &str) -> Option<String> {
    // Equivalent to your Nushell regex:
    // parse --regex '(?P<word>.*)\s+(?P<w2>\S+)\s+(?P<w1>\S+)'
    //
    // i.e. "word" = everything except the last two tokens.
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let phrase = parts[..parts.len() - 2].join(" ");
    if phrase.is_empty() {
        None
    } else {
        Some(phrase)
    }
}

fn random_phrase() -> Option<&'static str> {
    if PHRASES.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    PHRASES.choose(&mut rng).map(|s| s.as_str())
}

fn handle_client(mut stream: UnixStream) {
    // Zero-parameter protocol: ignore any input, just send one random phrase.
    match random_phrase() {
        Some(phrase) => {
            if let Err(e) = writeln!(stream, "{}", phrase) {
                eprintln!("Error writing to client: {}", e);
            }
        }
        None => {
            let _ = writeln!(stream, "NO_PHRASES_AVAILABLE");
        }
    }
}

fn main() -> std::io::Result<()> {
    let dict = get_dict_name();
    let socket_path = get_socket_path(&dict);

    // Clean up any stale socket.
    if Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    eprintln!("random_phrase_service listening on {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}

