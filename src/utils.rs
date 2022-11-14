use std::{env, fs};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use crate::manifest::Manifest;

pub fn run_command(command: &mut Command) -> String {
    let output = command
        .output()
        .expect("Failed to run command line");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        println!("stdout:\n{}", stdout);
        panic!("{}", stderr);
    }
    stdout
}

pub fn write_output(output: String, path: &str, filename: &str) -> String
{
    let current_dir = env::current_dir().expect("Could not find current directory");
    let path = format!("{}/{}/rtm/{}{}", current_dir.display(), path, filename, ".rtm");
    if !Path::new(&path).exists()
    {
        File::create(path.clone()).expect("Could not create a new file");
    }
    let mut file = match OpenOptions::new().write(true).truncate(true).open(path.clone())
    {
        Ok( f) => {f}
        Err(_) => { panic!("Could not access path {}", path); }
    };
    file.write_all(output.as_bytes())
        .expect("Could not output rtm");

    path
}

pub fn create_dir(path: &str)
{
    let mut path_string = String::from(path);
    let last_char = path_string.pop().unwrap();
    match last_char
    {
        '/' => { path_string.push(last_char) }
        _ =>
            {
                path_string.push(last_char);
                path_string.push('/');
            }
    }
    let new_path = format!("{}{}", path_string, "rtm");
    fs::create_dir_all(&new_path).expect("Something went wrong when trying to create rtm path");
}

pub fn run_manifest(manifest: Manifest, path: &str, name: &str) -> String
{
    let output = manifest.build();
    let path = write_output(output, path, name);
    run_command(Command::new("resim")
        .arg("run")
        .arg(path))
}