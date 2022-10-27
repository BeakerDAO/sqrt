use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use transaction::model::{Instruction, MethodIdentifier, TransactionManifest};
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

pub fn write_output(output: String, filename: &str) -> String
{
    let current_dir = env::current_dir().expect("Could not find current directory");
    let path = format!("{}/rtm/{}{}", current_dir.display(), filename, ".rtm");

    if !Path::new(&path).exists()
    {
        File::create(path.clone()).expect("Could not create a new file");
        let mut file = match OpenOptions::new().write(true).append(true).open(path.clone())
        {
            Ok( f) => {f}
            Err(_) => { panic!("Could not access path {}", path); }
        };
        file.write_all(output.as_bytes())
            .expect("Could not output rtm");
    }

    path
}

pub fn run_manifest(manifest: Manifest, name: &str)
{
    let output = manifest.build();
    let path = write_output(output, name);
    run_command(Command::new("resim")
        .arg("run")
        .arg(path));
}