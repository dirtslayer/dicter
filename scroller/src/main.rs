use std::env;
use std::io::{self, BufRead};
use std::thread;
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

const PAD: &str = "\u{00A0}";
const LHALF: &str = "\u{258C}"; // ▌ LEFT HALF BLOCK
const RHALF: &str = "\u{2590}"; // ▐ RIGHT HALF BLOCK

fn main() {
    // Expect flags: --width <n> --interval_ms <millis> [--duration_ms <millis>] [--chop <n>] [--start-delay <secs>] [--switch-delay <secs>] [--mode <mode>]
    let args: Vec<String> = env::args().collect();

    let mut width: Option<usize> = None;
    let mut interval_ms: Option<u64> = None;
    let mut duration_ms: Option<u64> = None;
    let mut chop: Option<usize> = None;
    let mut start_delay_secs: Option<u64> = None;
    let mut switch_delay_ms: Option<u64> = None;
    let mut mode: Option<String> = None;
    let mut lpad: Option<String> = None;
    let mut rpad: Option<String> = None;
    let mut nopad = false;
    let mut x: Option<usize> = None;
    let mut y: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--width" => {
                i += 1;
                width = args
                    .get(i)
                    .map(|s| s.parse().expect("width must be a positive integer"));
            }
            "--interval_ms" => {
                i += 1;
                interval_ms = args.get(i).map(|s| {
                    s.parse()
                        .expect("interval_ms must be a positive integer (milliseconds)")
                });
            }
            "--duration_ms" => {
                i += 1;
                duration_ms = args.get(i).map(|s| {
                    s.parse()
                        .expect("duration_ms must be a positive integer (milliseconds)")
                });
            }
            "--chop" => {
                i += 1;
                chop = args
                    .get(i)
                    .map(|s| s.parse().expect("chop must be a non-negative integer"));
            }
            "--delay" | "--start-delay" => {
                i += 1;
                start_delay_secs = args.get(i).map(|s| {
                    s.parse()
                        .expect("start-delay must be a non-negative integer (seconds)")
                });
            }
            "--switch-delay" => {
                i += 1;
                switch_delay_ms = args.get(i).map(|s| {
                    s.parse()
                        .expect("switch-delay must be a non-negative integer (milliseconds)")
                });
            }
            "--mode" => {
                i += 1;
                mode = args.get(i).cloned();
            }
            "--lpad" => {
                i += 1;
                lpad = args.get(i).cloned();
            }
            "--rpad" => {
                i += 1;
                rpad = args.get(i).cloned();
            }
            "--nopad" => {
                nopad = true;
            }
            "--x" => {
                i += 1;
                x = args
                    .get(i)
                    .map(|s| s.parse().expect("x must be a positive integer"));
            }
            "--y" => {
                i += 1;
                y = args
                    .get(i)
                    .map(|s| s.parse().expect("y must be a positive integer"));
            }
            other => {
                eprintln!("Unknown argument: {other}");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let width = width.unwrap_or_else(|| {
        eprintln!("Missing required flag: --width <n>");
        std::process::exit(1);
    });
    let interval_ms = interval_ms.unwrap_or_else(|| {
        eprintln!("Missing required flag: --interval_ms <milliseconds>");
        std::process::exit(1);
    });
    if mode != Some("reader".to_string()) && duration_ms.is_none() {
        eprintln!("Missing required flag: --duration_ms <milliseconds>");
        std::process::exit(1);
    }
    let chop = chop.unwrap_or(5);
    let start_delay_secs = start_delay_secs.unwrap_or(2);
    let switch_delay_ms = switch_delay_ms.unwrap_or(0);
    let mode = mode.unwrap_or_else(|| "dicter-default".to_string());

    let interval = Duration::from_millis(interval_ms);
    let start_delay = Duration::from_secs(start_delay_secs);
    let switch_delay = Duration::from_millis(switch_delay_ms);
    let lpad = if nopad {
        String::new()
    } else {
        lpad.unwrap_or_else(|| format!("{LHALF}{PAD}"))
    };
    let rpad = if nopad {
        String::new()
    } else {
        rpad.unwrap_or_else(|| format!("{PAD}{RHALF}"))
    };

    let cursor_prefix = match (x, y) {
        (Some(col), Some(row)) => format!("\x1B[{};{}H", row, col),
        (Some(col), None) => format!("\x1B[{}G", col),
        (None, Some(row)) => format!("\x1B[{}d", row),
        (None, None) => String::new(),
    };

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    if mode == "reader" {
        scroll_reader_main(width, &lpad, &rpad, &cursor_prefix, interval, &mut lines);
    } else {
        let duration = Duration::from_millis(duration_ms.unwrap());
        for line_result in lines {
            let line = match line_result {
                Ok(l) => l.trim().replace(' ', PAD),
                Err(_) => break,
            };

            match_mode(
                &line,
                width,
                chop,
                &lpad,
                &rpad,
                &cursor_prefix,
                start_delay,
                switch_delay,
                interval,
                duration,
                &mode,
            );
        }
    }
}

fn match_mode(
    line: &str,
    width: usize,
    chop: usize,
    lpad: &str,
    rpad: &str,
    cursor_prefix: &str,
    start_delay: Duration,
    switch_delay: Duration,
    interval: Duration,
    duration: Duration,
    mode: &str,
) {
    let lpw = lpad.width();
    let rpw = rpad.width();
    let inner_width = width.saturating_sub(lpw + rpw);
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        println!("{cursor_prefix}{lpad}{}{rpad}", PAD.repeat(inner_width));
        return;
    }

    match mode {
        "dicter-default" => scroll_dicter_default(
            line,
            chars,
            inner_width,
            chop,
            lpad,
            rpad,
            cursor_prefix,
            start_delay,
            switch_delay,
            interval,
            duration,
        ),
        "inner-bounce" => scroll_inner_bounce(
            line,
            chars,
            inner_width,
            lpad,
            rpad,
            cursor_prefix,
            start_delay,
            switch_delay,
            interval,
            duration,
        ),
        "marquee" => scroll_marquee(
            &line,
            inner_width,
            lpad,
            rpad,
            cursor_prefix,
            start_delay,
            switch_delay,
            interval,
            duration,
        ),
        _ => {
            eprintln!("Unknown mode: {mode}");
            std::process::exit(1);
        }
    }
}

fn scroll_dicter_default(
    line: &str,
    chars: Vec<char>,
    inner_width: usize,
    _chop: usize,
    lpad: &str,
    rpad: &str,
    cursor_prefix: &str,
    start_delay: Duration,
    switch_delay: Duration,
    interval: Duration,
    duration: Duration,
) {
    let total_width: usize = chars.iter().map(|c| c.width().unwrap_or(0)).sum();

    if total_width <= inner_width {
        let pad = inner_width - total_width;
        println!(
            "{cursor_prefix}{lpad}{}{line}{}{rpad}",
            PAD.repeat(pad),
            PAD.repeat(pad)
        );
        return;
    }

    scroll_inner_bounce(
        line,
        chars,
        inner_width,
        lpad,
        rpad,
        cursor_prefix,
        start_delay,
        switch_delay,
        interval,
        duration,
    );
}

fn scroll_inner_bounce(
    line: &str,
    _chars: Vec<char>,
    inner_width: usize,
    lpad: &str,
    rpad: &str,
    cursor_prefix: &str,
    start_delay: Duration,
    switch_delay: Duration,
    interval: Duration,
    duration: Duration,
) {
    let total_width = line.width();

    let start_time = Instant::now();

    if total_width <= inner_width {
        let max_offset = inner_width - total_width;

        let mut left_pad = 0usize;
        let mut moving_right = true;

        thread::sleep(start_delay);

        while start_time.elapsed() < duration {
            let right_pad = max_offset - left_pad;
            println!(
                "{cursor_prefix}{lpad}{}{line}{}{rpad}",
                PAD.repeat(left_pad),
                PAD.repeat(right_pad)
            );

            if !switch_delay.is_zero() && (left_pad == 0 || left_pad == max_offset) {
                thread::sleep(switch_delay);
            }

            thread::sleep(interval);

            if moving_right {
                if left_pad >= max_offset {
                    moving_right = false;
                } else {
                    left_pad += 1;
                }
            } else {
                if left_pad == 0 {
                    moving_right = true;
                } else {
                    left_pad -= 1;
                }
            }
        }
    } else {
        let max_offset = total_width - inner_width;
        let chars: Vec<char> = line.chars().collect();
        let total_chars = chars.len();

        let mut offset = 0usize;
        let mut moving_right = true;

        thread::sleep(start_delay);

        while start_time.elapsed() < duration {
            let start_idx = offset;
            let end_idx = (start_idx + inner_width).min(total_chars);
            let s: String = chars[start_idx..end_idx].iter().collect();
            let w = s.width();
            let right_pad = PAD.repeat(inner_width.saturating_sub(w));

            println!("{cursor_prefix}{lpad}{s}{right_pad}{rpad}");

            if !switch_delay.is_zero() && (offset == 0 || offset == max_offset) {
                thread::sleep(switch_delay);
            }

            thread::sleep(interval);

            if moving_right {
                if offset >= max_offset {
                    moving_right = false;
                } else {
                    offset += 1;
                }
            } else {
                if offset == 0 {
                    moving_right = true;
                } else {
                    offset -= 1;
                }
            }
        }
    }
}

fn scroll_reader_main<L>(
    width: usize,
    lpad: &str,
    rpad: &str,
    cursor_prefix: &str,
    interval: Duration,
    lines: &mut L,
) where
    L: Iterator<Item = Result<String, std::io::Error>>,
{
    let lpw = lpad.width();
    let rpw = rpad.width();
    let inner_width = width.saturating_sub(lpw + rpw);

    let mut window = vec![String::new(), String::new(), String::new()];
    let mut offset = 0usize;

    if let Some(Ok(line)) = lines.next() {
        window[2] = line.trim().replace(' ', PAD);
    }

    loop {
        let scroll_line = &window[1];
        let next_line = &window[2];

        let scroll_chars: Vec<char> = scroll_line.chars().collect();
        let next_chars: Vec<char> = next_line.chars().collect();

        let scroll_width: usize = scroll_chars.iter().map(|c| c.width().unwrap_or(0)).sum();
        let next_width: usize = next_chars.iter().map(|c| c.width().unwrap_or(0)).sum();

        let combined: Vec<char> = scroll_chars
            .iter()
            .chain(next_chars.iter())
            .copied()
            .collect();
        let combined_width = scroll_width + next_width;

        if combined_width == 0 {
            break;
        }

        let start_pos = offset;
        let end_pos = (start_pos + inner_width).min(combined_width);

        let mut pos = 0usize;
        let mut output = String::new();
        for ch in &combined {
            let ch_width = ch.width().unwrap_or(0);
            if pos >= start_pos && pos < end_pos {
                output.push(*ch);
            }
            if pos >= end_pos {
                break;
            }
            pos += ch_width;
        }

        let output_width = output.width();
        let right_pad = PAD.repeat(inner_width.saturating_sub(output_width));
        println!("{cursor_prefix}{lpad}{output}{right_pad}{rpad}");

        offset += 1;

        if offset >= scroll_width {
            window[1] = window[2].clone();

            if let Some(Ok(line)) = lines.next() {
                window[2] = line.trim().replace(' ', PAD);
            } else {
                window[2] = String::new();
            }

            offset = 0;
        }

        if window[1].is_empty() && window[2].is_empty() {
            break;
        }

        thread::sleep(interval);
    }

    for _ in 0..2 {
        println!("{cursor_prefix}{lpad}{}{rpad}", PAD.repeat(inner_width));
        thread::sleep(interval);
    }
}

fn scroll_marquee(
    line: &str,
    inner_width: usize,
    lpad: &str,
    rpad: &str,
    cursor_prefix: &str,
    start_delay: Duration,
    switch_delay: Duration,
    interval: Duration,
    duration: Duration,
) {
    let marquee_pad = PAD.repeat(inner_width);
    let marquee_text = format!("{}{}{}", marquee_pad, line, marquee_pad);
    let chars: Vec<char> = marquee_text.chars().collect();
    let total_chars = chars.len();

    let start_time = Instant::now();

    println!("{cursor_prefix}{lpad}{}{rpad}", marquee_pad);
    thread::sleep(start_delay);

    let max_start = total_chars - inner_width;

    while start_time.elapsed() < duration {
        for start_idx in 1..=max_start {
            if start_time.elapsed() >= duration {
                return;
            }

            let end_idx = (start_idx + inner_width).min(total_chars);
            let s: String = chars[start_idx..end_idx].iter().collect();
            let w = s.width();
            let right_pad = PAD.repeat(inner_width.saturating_sub(w));

            println!("{cursor_prefix}{lpad}{s}{right_pad}{rpad}");

            if !switch_delay.is_zero() && start_idx == max_start {
                thread::sleep(switch_delay);
            }

            thread::sleep(interval);
        }
    }
}
