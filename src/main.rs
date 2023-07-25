use chrono::prelude::*;
use chrono::Utc;
use clap::Parser;
use colored::*;
use std::io::Write;
use std::{env, fs, io, path::Path, process, time::SystemTime};

// Key for Env variable used to store the path to the Downloads folder.
const DOWNLOADS: &str = "Downloads";
const SECS_IN_A_DAY: u64 = 60 * 60 * 24;
const DELETE_COMMAND: &str = "DEL";
const DATE_FORMAT: &str = "%Y-%m-%d";
const DEFAULT_NUMBER_OF_DAYS: u64 = 600;

#[derive(Parser)]
#[command(name = "clndir")]
#[command(version = "1.0")]
#[command(
    about = "Cleans old files from a directory. It defaults to the value of the ENV var 'downloads'.\nProgram will return an error if no directory is specified and the ENV var is missing.\nProgram will ask user to confirm list of files unless --nowarn is specified.\nOnly files older than --age days will be deleted.\nFiles matching the pattern specified by --SKIP will not be deleted. This parameter can be repeated."
)]
struct Cli {
    #[arg(short, long)]
    dir: Option<String>,
    #[arg(short, long, default_value_t = DEFAULT_NUMBER_OF_DAYS)]
    age: u64,
    #[arg(short, long)]
    nowarn: bool,
    #[arg(short, long)]
    skip: Vec<String>,
}

#[derive(Debug)]
struct FileWithModifiedTime {
    name: String,
    modified_time: SystemTime,
}

fn main() {
    let cli = Cli::parse();

    let dir = match cli.dir {
        Some(dir) => dir,
        _ => match read_env_variable(DOWNLOADS) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        },
    };

    let exit_code = clean_dir(&dir, cli.age, cli.skip, cli.nowarn);
    match exit_code {
        Ok(_) => process::exit(0),
        Err(e) => {
            println!("Error : {}\n", e.to_string());
            process::exit(1);
        }
    }
}

fn read_env_variable(var_name: &str) -> Result<String, String> {
    match env::var(var_name) {
        Ok(value) => Ok(value),
        Err(_) => Err(format!("Environment variable {} not found", var_name)),
    }
}
fn clean_dir(
    dir: &str,
    age: u64,
    skip: Vec<String>,
    nowarn: bool,
) -> Result<u8, Box<dyn std::error::Error>> {
    match list_files_with_modified_time(dir) {
        Ok(files) => {
            match_and_delete(dir, files, age, skip, nowarn);
            Ok(0)
        }
        Err(e) => {
            eprintln!("\nDirectory name : {}", dir);
            Err(Box::new(e))
        }
    }
}

fn match_and_delete(
    dir: &str,
    files: Vec<FileWithModifiedTime>,
    age: u64,
    skip: Vec<String>,
    nowarn: bool,
) {
    let mut files_to_delete: Vec<FileWithModifiedTime> = Vec::new();

    for file in files {
        if is_file_ok_to_delete(&file, age, &skip) {
            files_to_delete.push(file);
        }
    }
    let do_delete;
    if !nowarn {
        do_delete = is_list_confirmed(&files_to_delete);
    } else {
        do_delete = true;
    }
    if do_delete {
        println!(
            "{} File(s) deleted",
            delete_files_in_directory(dir, &files_to_delete)
        );
    }
}
fn is_file_ok_to_delete(file: &FileWithModifiedTime, age: u64, skip: &Vec<String>) -> bool {
    if file.modified_time.elapsed().unwrap().as_secs() / (SECS_IN_A_DAY) < age {
        return false;
    }
    if skip.is_empty() {
        return true;
    } else {
        for pattern in skip {
            if file.name.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }
        true
    }
}
fn is_list_confirmed(files: &Vec<FileWithModifiedTime>) -> bool {
    display_files(files);

    let mut buffer = String::new();
    print!("\nType {} to delete these files : ", DELETE_COMMAND.red());
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut buffer);

    if buffer.trim() == DELETE_COMMAND {
        return true;
    } else {
        println!("Deletion canceled by user");
        false
    }
}
fn display_files(files: &Vec<FileWithModifiedTime>) {
    for file in files {
        let date_time = DateTime::<Utc>::from(file.modified_time);
        println!(
            "Last Modified {} - {} ",
            date_time.format(DATE_FORMAT).to_string().green(),
            file.name.yellow(),
        )
    }
}

fn list_files_with_modified_time(
    directory_path: &str,
) -> Result<Vec<FileWithModifiedTime>, io::Error> {
    let directory = Path::new(directory_path);

    let mut files_with_modified_time = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let modified_time = entry.metadata()?.modified()?;
            files_with_modified_time.push(FileWithModifiedTime {
                name,
                modified_time,
            });
        }
    }

    Ok(files_with_modified_time)
}

fn delete_files_in_directory(directory_path: &str, files: &Vec<FileWithModifiedTime>) -> u32 {
    let mut count = 0;
    for file in files {
        let file_path = Path::new(directory_path).join(&file.name);
        if let Err(e) = fs::remove_file(&file_path) {
            eprintln!("Error deleting file {}: {}", file.name.yellow(), e);
        } else {
            count += 1;
        }
    }
    count
}
