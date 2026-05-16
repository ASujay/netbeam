use std::{io, process::exit};


pub fn get_hostname() -> io::Result<String> {
    let hostname = hostname::get()?.to_string_lossy().to_string();
    Ok(hostname)
}

pub fn usage_err() {
    eprintln!("Usage: netbeam send <file_name> | netbeam receive");
    exit(1);
}

pub fn invalid_cmd_err(cmd: &String) {
    eprintln!("Invalid command: {}", cmd);
    exit(1);
}