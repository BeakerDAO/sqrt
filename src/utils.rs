use crate::manifest::Manifest;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

pub fn run_command(command: &mut Command, is_transaction: bool) -> String {
    let output = command.output().expect("Failed to run command line");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    lazy_static! {
        static ref SUCCESS_RE: Regex = Regex::new("Transaction Status: COMMITTED SUCCESS").unwrap();
    }

    if is_transaction && !SUCCESS_RE.is_match(&stdout) {
        panic!("stdout:\n{}", stdout);
    }

    if !output.status.success() {
        println!("stdout:\n{}", stdout);
        panic!("{}", stderr);
    }
    stdout
}

pub fn write_output(output: String, path: &str, filename: &str) -> String {
    let current_dir = env::current_dir().expect("Could not find current directory");
    let path = format!(
        "{}/{}/rtm/{}{}",
        current_dir.display(),
        path,
        filename,
        ".rtm"
    );
    if !Path::new(&path).exists() {
        File::create(path.clone()).expect("Could not create a new file");
    }
    let mut file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path.clone())
    {
        Ok(f) => f,
        Err(_) => {
            panic!("Could not access path {}", path);
        }
    };
    file.write_all(output.as_bytes())
        .expect("Could not output rtm");

    path
}

pub fn create_dir(path: &str) {
    let mut path_string = String::from(path);
    let last_char = path_string.pop().unwrap();
    match last_char {
        '/' => path_string.push(last_char),
        _ => {
            path_string.push(last_char);
            path_string.push('/');
        }
    }
    let new_path = format!("{}{}", path_string, "rtm");
    fs::create_dir_all(&new_path).expect("Something went wrong when trying to create rtm path");
}

pub fn run_manifest(manifest: Manifest, path: &str, name: &str) -> String {
    let output = manifest.build();
    let path = write_output(output, path, name);
    run_command(Command::new("resim").arg("run").arg(path), true)
}

pub fn transfer(from: &str, to: &str, asset: &str, amount: &str) -> String {
    run_command(
        Command::new("resim")
            .arg("run")
            .arg("rtm/transfer.rtm")
            .env("account1", from)
            .env("account2", to)
            .env("asset", asset)
            .env("amount", amount),
        true,
    )
}

pub fn write_transfer(path: &str) {
    let transfer_str = String::from(
        r#"CALL_METHOD
    ComponentAddress("${account1}")
    "lock_fee"
    Decimal("100");

CALL_METHOD
    ComponentAddress("${account1}")
    "withdraw_by_amount"
    Decimal("${amount}")
    ResourceAddress("${asset}");

TAKE_FROM_WORKTOP_BY_AMOUNT
    Decimal("${amount}")
    ResourceAddress("${asset}")
    Bucket("Asset");

CALL_METHOD
    ComponentAddress("${account2}")
    "deposit"
    Bucket("Asset");

CALL_METHOD
    ComponentAddress("${account1}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");"#,
    );

    write_output(transfer_str, path, "transfer");
}

pub fn generate_owner_badge() -> String
{
    let output = run_command(Command::new("resim").arg("new-simple-badge"), false);

    lazy_static! {
        static ref NFADDRESS_RE: Regex = Regex::new(r#"NFAddress: (.*)"#).unwrap();
    }

    let badge_address = &NFADDRESS_RE.captures(&output).expect("Unexpected error")[1];
    String::from(badge_address)
}
