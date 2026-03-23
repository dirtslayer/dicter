use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::env;
use std::process::Command;

fn debug(msg: &str) {
    if std::env::var("DEBUG").is_ok() {
        eprintln!("[DEBUG] {}", msg);
    }
}

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

fn dict_to_dictd_name(dict: &str) -> String {
    dict.replace("freedict", "fd")
}

fn get_interval() -> u64 {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--interval" {
            if let Some(val) = args.next() {
                if let Ok(secs) = val.parse::<u64>() {
                    return secs;
                }
            }
        }
    }
    10
}

fn get_pronounce() -> bool {
    env::args().any(|a| a == "--pronounce")
}

fn strip_pronunciation_segments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_seg = false;
    let mut replaced = false;

    for ch in s.chars() {
        if ch == '/' {
            if in_seg {
                in_seg = false;
                if !replaced {
                    out.push('-');
                    replaced = true;
                }
            } else {
                in_seg = true;
                replaced = false;
            }
            continue;
        }

        if !in_seg {
            out.push(ch);
        }
    }

    // If we had a trailing unmatched '/', drop it silently.
    // Normalize whitespace a bit so "word  -  def" becomes "word - def".
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn ensure_randphrase_running(dict: &str) {
    let socket_path = format!("/tmp/phrases-{}.sock", dict);
    
    // Try to connect to existing socket
    debug(&format!("Checking if randphrase is running on {}", socket_path));
    if UnixStream::connect(&socket_path).is_ok() {
        debug("randphrase is already running");
        return;
    }
    
    // Socket doesn't exist or isn't responding, try to start randphrase
    debug(&format!("Starting randphrase with --dict {}", dict));
    match Command::new("randphrase")
        .arg("--dict")
        .arg(dict)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => {
            debug("randphrase started, waiting for socket to be ready…");
            // Give it time to start and create the socket
            thread::sleep(Duration::from_millis(1000));
            
            // Verify it's ready
            if UnixStream::connect(&socket_path).is_ok() {
                debug("randphrase is ready");
                return;
            } else {
                eprintln!("Error: randphrase started but socket did not become ready");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: Could not start randphrase: {}", e);
            eprintln!("Make sure randphrase is in your PATH or activate mise.toml");
            std::process::exit(1);
        }
    }
}

fn get_rand_phrase(dict: &str) -> std::io::Result<String> {
    debug("Connecting to randphrase service…");
    let socket_path = format!("/tmp/phrases-{}.sock", dict);
    let stream = UnixStream::connect(&socket_path)?;
    let mut reader = BufReader::new(stream);

    let mut line = String::new();
    debug("Reading phrase…");
    reader.read_line(&mut line)?;
    let phrase = line.trim().to_string();
    debug(&format!("Got phrase: {}", phrase));

    Ok(phrase)
}

fn extract_word_from_match(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line.rfind('"')?;
    if end > start {
        Some(line[start+1 .. end].to_string())
    } else {
        None
    }
}

fn dict_match_exact_first(phrase: &str, dict: &str) -> std::io::Result<Option<String>> {
    debug("Connecting to dictd on port 2628…");
    let mut stream = TcpStream::connect(("127.0.0.1", 2628))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let dictd_dict = dict_to_dictd_name(dict);
    let command = format!("MATCH {} exact \"{}\"", dictd_dict, phrase);
    debug(&format!("Sending MATCH command: {}", command));
    writeln!(stream, "{}", command)?;

    let mut line = String::new();

    debug("Reading MATCH response…");
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            debug("MATCH: EOF reached");
            break;
        }

        let trimmed = line.trim_end();
        debug(&format!("MATCH line: {:?}", trimmed));

        if trimmed == "." {
            debug("MATCH: terminating dot found");
            break;
        }

        if trimmed.starts_with(char::is_numeric) {
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if let Some(word) = extract_word_from_match(trimmed) {
            debug(&format!("MATCH: definable word = {}", word));
            return Ok(Some(word));
        } else {
            debug("MATCH: could not parse definable word");
        }
    }

    Ok(None)
}

fn dict_define(word: &str, dict: &str) -> std::io::Result<Vec<String>> {
    debug("Connecting to dictd for DEFINE…");
    let mut stream = TcpStream::connect(("127.0.0.1", 2628))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let dictd_dict = dict_to_dictd_name(dict);
    let command = format!("DEFINE {} \"{}\"", dictd_dict, word);
    debug(&format!("Sending DEFINE command: {}", command));
    writeln!(stream, "{}", command)?;

    let mut defs = Vec::new();
    let mut line = String::new();

    debug("Reading DEFINE response…");
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            debug("DEFINE: EOF reached");
            break;
        }

        let trimmed = line.trim_end();
        debug(&format!("DEFINE line: {:?}", trimmed));

        if trimmed == "." {
            debug("DEFINE: terminating dot found");
            break;
        }

        // Skip protocol response lines (e.g., "150 ...", "220 ...", "250 ...")
        // These start with 3 digits followed by space
        if trimmed.len() > 3 && trimmed.chars().next().unwrap().is_numeric() {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if !parts.is_empty() && parts[0].chars().all(|c| c.is_numeric()) {
                continue;
            }
        }

        // Skip lines that are only metadata (contain quotes and dictionary name)
        if trimmed.contains("\"") && trimmed.contains("fd-") {
            continue;
        }

        // Remove leading numbering like "1. " or "2. "
        let cleaned = if let Some(dot_pos) = trimmed.find('.') {
            let prefix = &trimmed[..dot_pos];
            if prefix.trim().chars().all(|c| c.is_numeric()) {
                trimmed[dot_pos + 1..].trim_start().to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            trimmed.to_string()
        };

        if !cleaned.is_empty() {
            defs.push(cleaned);
        }
    }

    Ok(defs)
}
fn main() -> std::io::Result<()> {
    let dict = get_dict_name();
    let interval = get_interval();
    let pronounce = get_pronounce();

    eprintln!("Using dictionary: {}", dict);
    eprintln!("Using interval: {} seconds", interval);
    eprintln!("Using pronounce: {}", pronounce);
    
    // Ensure randphrase is running
    ensure_randphrase_running(&dict);

    loop {
        let phrase = get_rand_phrase(&dict)?;
        // println!("Random phrase: {}", phrase);

        let first_match = dict_match_exact_first(&phrase, &dict)?;

        match first_match {
            Some(m) => {
                // println!("First match: {}", m);

                let defs = dict_define(&m, &dict)?;
                if defs.is_empty() {
                    println!("No definition found.");
                } else {
                    let mut out = defs.join(" ").replace(',', "");
                    if !pronounce {
                        out = strip_pronunciation_segments(&out);
                    }
                    println!("{out}");
                }
            }
            None => println!("No exact match found."),
        }

        // Print a blank line to separate entries
        // println!();

        // Sleep for configured interval
        thread::sleep(Duration::from_secs(interval));
    }
}

