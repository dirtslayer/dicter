use std::io::{self, BufRead, Read, Write};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut linelen: usize = 80;
    let mut interval_secs: Option<u64> = None;
    let mut interval_ms: Option<u64> = None;
    let mut repeat: bool = true;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--linelen" => {
                i += 1;
                linelen = args
                    .get(i)
                    .unwrap_or_else(|| usage_die("--linelen requires a value"))
                    .parse()
                    .unwrap_or_else(|_| usage_die("--linelen must be a positive integer"));
            }
            "--interval" => {
                i += 1;
                interval_secs = Some(
                    args.get(i)
                        .unwrap_or_else(|| usage_die("--interval requires a value"))
                        .parse()
                        .unwrap_or_else(|_| {
                            usage_die("--interval must be a non-negative integer (seconds)")
                        }),
                );
            }
            "--interval_ms" => {
                i += 1;
                interval_ms = Some(
                    args.get(i)
                        .unwrap_or_else(|| usage_die("--interval_ms requires a value"))
                        .parse()
                        .unwrap_or_else(|_| {
                            usage_die("--interval_ms must be a non-negative integer (milliseconds)")
                        }),
                );
            }
            "--repeat" => {
                i += 1;
                repeat = args
                    .get(i)
                    .unwrap_or_else(|| usage_die("--repeat requires a value (true/false)"))
                    .parse()
                    .unwrap_or_else(|_| usage_die("--repeat must be true or false"));
            }
            "--no-repeat" => {
                repeat = false;
            }
            other => usage_die(&format!("Unknown argument: {other}")),
        }
        i += 1;
    }

    if linelen == 0 {
        usage_die("--linelen must be > 0");
    }

    let interval = if let Some(ms) = interval_ms {
        Duration::from_millis(ms)
    } else if let Some(s) = interval_secs {
        Duration::from_secs(s)
    } else {
        Duration::from_secs(10)
    };

    if repeat {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .unwrap_or_else(|e| usage_die(&format!("failed reading stdin: {e}")));
        if input.is_empty() {
            return;
        }
        let normalized: String = input
            .chars()
            .map(|c| match c {
                '\n' | '\r' => ' ',
                other => other,
            })
            .collect();
        let chars: Vec<char> = normalized.chars().collect();
        if chars.is_empty() {
            return;
        }
        let mut stdout = io::stdout().lock();
        let mut idx: usize = 0;
        loop {
            let end = (idx + linelen).min(chars.len());
            let chunk: String = chars[idx..end].iter().collect();
            if chunk.is_empty() {
                break;
            }
            writeln!(stdout, "{chunk}")
                .unwrap_or_else(|e| usage_die(&format!("failed writing stdout: {e}")));
            stdout.flush().ok();
            idx = end;
            if idx >= chars.len() {
                idx = 0;
            }
            std::thread::sleep(interval);
        }
        return;
    }

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout().lock();

    let mut buffer = String::with_capacity(linelen * 2);

    loop {
        if buffer.len() >= linelen {
            let chunk = &buffer[..linelen];
            if writeln!(stdout, "{chunk}").is_err() {
                break;
            }
            stdout.flush().ok();
            buffer.drain(..linelen);
            std::thread::sleep(interval);
            continue;
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                if !buffer.is_empty() {
                    if writeln!(stdout, "{}", buffer).is_err() {
                        break;
                    }
                    stdout.flush().ok();
                }
                break;
            }
            Ok(_) => {
                for c in line.chars() {
                    match c {
                        '\n' | '\r' => buffer.push(' '),
                        other => buffer.push(other),
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                if !buffer.is_empty() {
                    if writeln!(stdout, "{}", buffer).is_err() {
                        break;
                    }
                    stdout.flush().ok();
                }
                break;
            }
            Err(e) => usage_die(&format!("failed reading stdin: {e}")),
        }
    }
}

fn usage_die(msg: &str) -> ! {
    eprintln!("{msg}");
    eprintln!(
        "Usage: slowcat [--linelen <n>] [--interval <secs>] [--interval_ms <ms>] [--repeat <true|false>] [--no-repeat]"
    );
    std::process::exit(2);
}
