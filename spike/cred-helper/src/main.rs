//! Spike: a Git credential helper backed by the OS password store.

use keyring::Entry;
use std::io::{self, BufRead};

const SERVICE: &str = "ramoso-spike";
const USAGE: &str = "usage: cred-helper set <host> (token on stdin) | delete <host> | get";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        ["set", host] => set(host),
        ["delete", host] => delete(host),
        ["get"] => get(),

        [] => eprintln!("{USAGE}"),

        // Other operations (Git's `store`, `erase`, future ones) are ignored, as Git requires.
        _ => {}
    }
}

/// Saves a token for a host; the token comes on stdin, never as an argument (shell history, `ps`).
fn set(host: &str) {
    // First line of stdin, without the newline.
    let mut token = String::new();
    io::stdin().read_line(&mut token).expect("reading the token from stdin failed");
    let token = token.trim_end_matches(['\r', '\n']);
    assert!(!token.is_empty(), "no token on stdin");

    Entry::new(SERVICE, host).and_then(|e| e.set_password(token)).expect("store failed");
    eprintln!("saved token for {host}");
}

/// Removes a host's token.
fn delete(host: &str) {
    Entry::new(SERVICE, host).and_then(|e| e.delete_credential()).expect("delete failed");
    eprintln!("deleted token for {host}");
}

/// Answers Git: reads key=value lines, prints the token for the host.
fn get() {
    // Input ends at a blank line or EOF.
    let host = io::stdin()
        .lock()
        .lines()
        .map_while(Result::ok)
        .take_while(|line| !line.is_empty())
        .find_map(|line| line.strip_prefix("host=").map(str::to_owned));

    // Look the host up in the password store.
    let token = host.as_deref().map(|host| Entry::new(SERVICE, host).and_then(|e| e.get_password()));

    // No token: `quit=1` stops Git asking other sources.
    match token {
        Some(Ok(token)) => print!("username=ramoso\npassword={token}\n"),
        Some(Err(error)) => {
            eprintln!("cred-helper: no token for {}: {error}", host.unwrap_or_default());
            print!("quit=1\n");
        }
        None => print!("quit=1\n"),
    }
}
