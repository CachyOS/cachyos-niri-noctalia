use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::Write;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

#[derive(Parser)]
#[command(name = "niri-clipboard")]
#[command(about = "Clipboard history manager for Niri/Wayland")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the clipboard store daemon (monitors clipboard)
    Store,
    /// Show clipboard history picker and paste selection
    Pick,
    /// Wipe clipboard history
    Wipe,
    /// Show clipboard history list
    List,
}

fn get_config_dir() -> String {
    std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap()))
}

fn get_style_path() -> String {
    format!("{}/wofi/style.css", get_config_dir())
}

fn run_cmd(cmd: &str, args: &[&str]) -> String {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap_or_else(|e| panic!("Failed to run {}: {}", cmd, e));

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_cmd_stdin(cmd: &str, args: &[&str], input: &[u8]) -> String {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to run {}: {}", cmd, e));

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(input).ok();
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap_or_else(|e| panic!("Failed to wait for {}: {}", cmd, e));
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn classify_content(data: &[u8]) -> String {
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(data).unwrap();
    let path = tmp.path().to_str().unwrap();

    let mime = run_cmd("file", &["--mime-type", "-b", path]);
    let mime = mime.trim().to_string();

    match mime.as_str() {
        m if m.starts_with("image/") => {
            let dims = run_cmd("identify", &["-format", "%wx%h", path]);
            let dims = dims.trim();
            if dims.is_empty() {
                "[IMG]".to_string()
            } else {
                format!("[IMG {}]", dims)
            }
        }
        m if m.starts_with("video/") => "[VIDEO]".to_string(),
        m if m.starts_with("audio/") => "[AUDIO]".to_string(),
        "application/pdf" => {
            let size = run_cmd("du", &["-h", path]);
            let size = size.split_whitespace().next().unwrap_or("?");
            format!("[PDF {}]", size)
        }
        m if m.starts_with("text/") || m == "application/json" || m == "application/xml" => {
            let preview = String::from_utf8_lossy(data);
            let preview: String = preview.chars().take(80).collect();
            let preview = preview.replace('\n', " ");
            format!("  ─ {}", preview)
        }
        _ => {
            let content = String::from_utf8_lossy(data).to_string();
            if std::path::Path::new(content.trim()).exists() {
                let fname = std::path::Path::new(content.trim())
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let fsize = run_cmd("du", &["-h", content.trim()]);
                let fsize = fsize.split_whitespace().next().unwrap_or("?");
                let fmime = run_cmd("file", &["--mime-type", "-b", content.trim()]);
                let fmime = fmime.trim();
                let tag = match fmime {
                    m if m.starts_with("image/") => "FILE:img",
                    m if m.starts_with("video/") => "FILE:vid",
                    m if m.starts_with("audio/") => "FILE:aud",
                    _ => "FILE",
                };
                format!("[{}] {} ({})", tag, fname, fsize)
            } else {
                let preview: String = content.chars().take(60).collect();
                let preview = preview.replace('\n', " ");
                format!("  ─ {}", preview)
            }
        }
    }
}

fn cmd_store() {
    eprintln!("niri-clipboard: starting clipboard store daemon");

    let mut text_child = Command::new("wl-paste")
        .args(["--type", "text", "--watch", "cliphist", "store"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start wl-paste text monitor");

    let mut image_child = Command::new("wl-paste")
        .args(["--type", "image", "--watch", "cliphist", "store"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start wl-paste image monitor");

    eprintln!("niri-clipboard: daemon running (text pid={}, image pid={})",
              text_child.id(), image_child.id());

    loop {
        if let Ok(Some(status)) = text_child.try_wait() {
            eprintln!("niri-clipboard: text monitor exited: {}", status);
            break;
        }
        if let Ok(Some(status)) = image_child.try_wait() {
            eprintln!("niri-clipboard: image monitor exited: {}", status);
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }

    text_child.kill().ok();
    image_child.kill().ok();
}

fn cmd_pick() {
    let list = run_cmd("cliphist", &["list"]);
    let items: Vec<String> = list.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    if items.is_empty() {
        eprintln!("niri-clipboard: clipboard history is empty");
        return;
    }

    // Build enhanced list for wofi display AND a map back to original cliphist keys
    // Key: wofi display line -> Value: original cliphist list line (the full key for decode)
    let mut key_map: HashMap<String, String> = HashMap::new();
    let mut enhanced = String::new();

    for item in &items {
        let decoded = run_cmd_stdin("cliphist", &["decode"], item.as_bytes());
        let tag = classify_content(decoded.as_bytes());

        let display_line = if tag.starts_with("  ─") {
            format!("{}{}", item, tag)
        } else {
            format!("{} {}", tag.trim(), item)
        };

        key_map.insert(display_line.clone(), item.clone());
        enhanced.push_str(&display_line);
        enhanced.push('\n');
    }

    // Show wofi picker
    let style_path = get_style_path();
    let mut wofi = Command::new("wofi")
        .args([
            "--show", "dmenu",
            "--prompt", "Clipboard History",
            "--width", "600",
            "--height", "400",
            "--style", &style_path,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start wofi");

    if let Some(ref mut stdin) = wofi.stdin {
        stdin.write_all(enhanced.as_bytes()).ok();
    }
    drop(wofi.stdin.take());

    let output = wofi.wait_with_output().expect("Failed to wait for wofi");
    let selected = String::from_utf8_lossy(&output.stdout).to_string();
    let selected = selected.trim().to_string();

    if selected.is_empty() {
        return;
    }

    // Look up the original cliphist key from our map
    let original_key = match key_map.get(&selected) {
        Some(key) => key.as_str(),
        None => {
            eprintln!("niri-clipboard: could not find original key for selection");
            return;
        }
    };

    // Decode and copy to clipboard
    let decoded = run_cmd_stdin("cliphist", &["decode"], original_key.as_bytes());

    // Write decoded content to temp file for reliable binary copy
    let mut clip_file = NamedTempFile::new().unwrap();
    clip_file.write_all(decoded.as_bytes()).unwrap();
    clip_file.flush().unwrap();
    let clip_path = clip_file.path().to_str().unwrap();

    // Copy to clipboard via file (more reliable than pipe for binary content)
    let copy_status = Command::new("sh")
        .args(["-c", &format!("cat '{}' | wl-copy", clip_path)])
        .status();

    match copy_status {
        Ok(s) if s.success() => {
            eprintln!("niri-clipboard: copied to clipboard");
        }
        Ok(s) => {
            eprintln!("niri-clipboard: wl-copy exited with {}", s);
            return;
        }
        Err(e) => {
            eprintln!("niri-clipboard: wl-copy failed: {}", e);
            return;
        }
    }

    // Auto-paste: -s sleeps AFTER connecting to Wayland but BEFORE sending keys,
    // giving wofi time to fully close and focus to return to the previous window.
    let wtype_status = Command::new("wtype")
        .args(["-s", "800", "-M", "ctrl", "-M", "shift", "v"])
        .status();

    match wtype_status {
        Ok(s) if s.success() => {
            eprintln!("niri-clipboard: pasted successfully");
        }
        Ok(s) => {
            eprintln!("niri-clipboard: wtype exited with {}", s);
        }
        Err(e) => {
            eprintln!("niri-clipboard: wtype failed: {}", e);
        }
    }
}

fn cmd_wipe() {
    let status = Command::new("cliphist")
        .arg("wipe")
        .status()
        .expect("Failed to run cliphist wipe");

    if status.success() {
        eprintln!("niri-clipboard: clipboard history wiped");
    } else {
        eprintln!("niri-clipboard: failed to wipe history");
    }
}

fn cmd_list() {
    let list = run_cmd("cliphist", &["list"]);
    if list.is_empty() {
        eprintln!("niri-clipboard: clipboard history is empty");
    } else {
        print!("{}", list);
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Store => cmd_store(),
        Commands::Pick => cmd_pick(),
        Commands::Wipe => cmd_wipe(),
        Commands::List => cmd_list(),
    }
}
