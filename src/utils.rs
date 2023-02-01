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

    if !is_transaction && !output.status.success() {
        println!("stdout:\n{}", stdout);
        panic!("{}", stderr);
    }
    else
    {
        stdout
    }
}

pub fn write_manifest(output: String, path: &str, filename: &str) -> String {
    let current_dir = env::current_dir().expect("Could not find current directory");
    let path = format!(
        "{}/{}/rtm/generated/{}{}",
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
    let custom_path = format!("{}{}", path_string, "rtm/custom");
    let generated_path = format!("{}{}", path_string, "rtm/generated");
    fs::create_dir_all(&custom_path)
        .expect("Something went wrong when trying to create custom rtm folder path");
    fs::create_dir_all(&generated_path)
        .expect("Something went wrong when trying to create generated rtm folder path");
}

pub fn run_manifest(
    package_path: &str,
    name: &str,
    custom_manifest: bool,
    env_variables_binding: Vec<(String, String)>
) -> (String, String) {
    let current_dir = env::current_dir().expect("Could not find current directory");
    let sub_folder = if custom_manifest {
        "custom"
    } else {
        "generated"
    };
    let path = format!(
        "{}/{}/rtm/{}/{}{}",
        current_dir.display(),
        package_path,
        sub_folder,
        name,
        ".rtm"
    );
    let manifest_output = manifest_called(package_path, name, custom_manifest, &env_variables_binding);

    let stdout = run_command(
        Command::new("resim")
            .arg("run")
            .arg(path)
            .envs(env_variables_binding),
        true,
    );

    (manifest_output, stdout)
}

fn manifest_called(
    package_path: &str,
    name: &str,
    custom_manifest: bool,
    env_variables_binding: &Vec<(String, String)>,
) -> String {
    let current_dir = env::current_dir().expect("Could not find current directory");
    let sub_folder = if custom_manifest {
        "custom"
    } else {
        "generated"
    };
    let path = format!(
        "{}/{}/rtm/{}/{}{}",
        current_dir.display(),
        package_path,
        sub_folder,
        name,
        ".rtm"
    );

    let mut manifest = fs::read_to_string(path).expect("Should have been able to read the file");
    for (arg_name, arg_value) in env_variables_binding {
        let gen_arg = format!("${{{}}}", arg_name);
        manifest = manifest.replace(gen_arg.as_str(), arg_value.as_str());
    }

    manifest
}

pub fn generated_manifest_exists(method_name: &str, path: &str) -> bool {
    let current_dir = env::current_dir().expect("Could not find current directory");
    let path = format!(
        "{}/{}/rtm/generated/{}{}",
        current_dir.display(),
        path,
        method_name,
        ".rtm"
    );
    Path::new(&path).exists()
}

pub fn generate_owner_badge() -> String {
    let output = run_command(Command::new("resim").arg("new-simple-badge"), false);

    lazy_static! {
        static ref NFADDRESS_RE: Regex = Regex::new(r#"NFAddress: (.*)"#).unwrap();
    }

    let badge_address = &NFADDRESS_RE.captures(&output).expect("Unexpected error")[1];
    String::from(badge_address)
}
