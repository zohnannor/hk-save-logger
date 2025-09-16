use std::{
    fmt,
    fs::{self, File},
    io::{self, Write as _},
    path::{Path, PathBuf},
    time::SystemTime,
};

use aes::{
    Aes256,
    cipher::{BlockDecryptMut as _, BlockEncryptMut as _, KeyInit as _, block_padding::Pkcs7},
};
use base64::prelude::*;
use clap::Parser;
use directories::BaseDirs;
use nano_leb128::ULEB128;
use owo_colors::OwoColorize as _;
use serde_json::Value;

const KEY: &[u8; 32] = b"UKu52ePUBwetZ9wNX88o54dnfKRu0T1l";
const CSHARP_HEADER: [u8; 22] = [
    0x00, 0x01, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x06, 0x01, 0x00, 0x00, 0x00,
];

fn get_modified_time(path: &Path) -> io::Result<Option<SystemTime>> {
    match fs::metadata(path) {
        Ok(metadata) => metadata.modified().map(Some),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn fmt(path: impl fmt::Display, old: impl fmt::Display, new: impl fmt::Display) -> String {
    format!("{path}: {old} -> {new}")
}

// Recursive function to compare two JSON values and track the path
fn compare_json(
    path: &str,
    old: &Value,
    new: &Value,
    changes: &mut Vec<String>,
    log_changes: &mut Vec<String>,
) {
    match (old, new) {
        (Value::Object(old_obj), Value::Object(new_obj)) => {
            // Check for keys in old but not in new (removed)
            for key in old_obj.keys() {
                if !new_obj.contains_key(key) {
                    let path = format!("{path}.{key}");
                    let old = &old_obj[key];
                    let new = "null";
                    changes.push(fmt(&path, old, new.red()));
                    log_changes.push(fmt(&path, old, new));
                }
            }

            // Check for keys in new but not in old (added)
            for key in new_obj.keys() {
                if !old_obj.contains_key(key) {
                    let path = format!("{path}.{key}");
                    let old = "null";
                    let new = &new_obj[key];
                    changes.push(fmt(&path, old.red(), new.green()));
                    log_changes.push(fmt(&path, old, new));
                }
            }

            // Check common keys
            for key in old_obj.keys().filter(|k| new_obj.contains_key(*k)) {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                compare_json(
                    &new_path,
                    &old_obj[key],
                    &new_obj[key],
                    changes,
                    log_changes,
                );
            }
        }
        (Value::Array(old_arr), Value::Array(new_arr)) => {
            // Simple approach: compare by index
            let max_len = old_arr.len().max(new_arr.len());
            for i in 0..max_len {
                let item_path = format!("{path}[{i}]");
                if i < old_arr.len() && i < new_arr.len() {
                    compare_json(&item_path, &old_arr[i], &new_arr[i], changes, log_changes);
                } else if i < old_arr.len() {
                    let old = &old_arr[i];
                    let new = "removed";
                    changes.push(fmt(&item_path, old.red(), new.red()));
                    log_changes.push(fmt(&item_path, old, new));
                } else {
                    let old = "null";
                    let new = &new_arr[i];
                    changes.push(fmt(&item_path, old.red(), new.green()));
                    log_changes.push(fmt(&item_path, old, new));
                }
            }
        }
        _ => {
            if old != new {
                changes.push(fmt(path, old.red(), new.green()));
                log_changes.push(fmt(path, old, new));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum Game {
    #[clap(alias = "hk")]
    HollowKnight,
    #[clap(alias = "ss")]
    Silksong,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HollowKnight => f.write_str("hollow-knight"),
            Self::Silksong => f.write_str("silksong"),
        }
    }
}

impl Game {
    fn savefile_path(self, save: u8) -> io::Result<PathBuf> {
        let savefile_path = if let Some(dirs) = BaseDirs::new() {
            let base = dirs.config_dir();

            #[cfg(target_os = "windows")]
            {
                // on windows, first go to LocalLow from Roaming
                let base = base.join("..").join("LocalLow");
                base.join("Team Cherry").join(match self {
                    Self::HollowKnight => "Hollow Knight",
                    Self::Silksong => "Hollow Knight Silksong",
                })
            }
            #[cfg(target_os = "macos")]
            {
                base.join(match self {
                    Self::HollowKnight => "unity.Team Cherry.Hollow Knight",
                    Self::Silksong => "unity.Team-Cherry.Silksong",
                })
            }
            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            {
                base.join("unity3d").join("Team Cherry").join(match self {
                    Self::HollowKnight => "Hollow Knight",
                    Self::Silksong => "Hollow Knight Silksong",
                })
            }
        } else {
            println!(
                "Couldn't get savefile directory on your system. \
                 Please specify the save file manually using the --path flag."
            );
            return Err(io::Error::other("Couldn't get savefile directory"));
        };

        // For Steam, each user’s save files will be in a sub-folder of their
        // Steam user ID. For non-Steam builds, save files will be in a default
        // sub-folder.
        // (c) Team Cherry

        let steam_userid_dir = fs::read_dir(&savefile_path)?.flatten().find_map(|entry| {
            (entry
                .file_name()
                .to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_digit())
                && entry.file_type().ok()?.is_dir())
            .then_some(entry.path())
        });

        let file_path = format!("user{save}.dat");
        Ok(steam_userid_dir.map_or_else(
            || savefile_path.join(&file_path),
            |steam_userid_dir| steam_userid_dir.join(&file_path),
        ))
    }

    fn from_path(savefile_path: &PathBuf) -> Self {
        if savefile_path.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s.contains("Silksong") || s.contains("silksong")
        }) {
            Self::Silksong
        } else {
            Self::HollowKnight
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Game to parse (hollow-knight/hk or silksong/ss)
    #[arg(required_unless_present = "path", conflicts_with = "path")]
    game: Option<Game>,
    /// Save slot to parse (1-4)
    #[arg(required_unless_present = "path", conflicts_with = "path")]
    save: Option<u8>,

    /// Decode save file (default) or encode (it won't place the file for you,
    /// you have to do it manually, this is done to prevent data loss. Rename
    /// file to user1.dat (or whatever number you need) and place it in the
    /// correct folder)
    #[arg(long)]
    encode: bool,

    /// Path to save file (default: auto-detect)
    #[arg(long, required_unless_present = "game", conflicts_with = "game")]
    path: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let Args {
        game,
        save,
        encode,
        path,
    } = Args::parse();

    // Determine the save file path
    let savefile_path = if let Some(path) = path {
        path
    } else {
        let game = game.expect("game is required when not using --path");
        let save = save.expect("save is required when not using --path");
        game.savefile_path(save)?
    };

    let game = game.unwrap_or_else(|| Game::from_path(&savefile_path));
    let save = save.map_or_else(
        || {
            savefile_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .strip_prefix("user")
                .unwrap_or_default()
                .strip_suffix(".dat")
                .unwrap_or_default()
                .parse()
                .expect("Couldn't parse save number")
        },
        |save| save,
    );

    if encode {
        let json = fs::read(format!("{game}-{save}.json")).unwrap();
        let dat = encrypt_save(&json);
        let path = format!("{game}-{save}.dat");
        fs::write(&path, &dat).unwrap();
        println!("Save file {path} encoded!");
        return Ok(());
    }

    println!("Using save file {}", savefile_path.display());
    let mut previous = fs::read_to_string(format!("{game}-{save}.json")).unwrap_or_default();
    let mut last_modified = None;
    let mut change_count = 0;
    let mut log_file = File::options()
        .append(true)
        .create(true)
        .open(format!("{game}-{save}.log"))?;

    loop {
        let current_modified = get_modified_time(&savefile_path)?;

        if current_modified != last_modified {
            match fs::read(&savefile_path) {
                Ok(dat) => {
                    let v = decrypt_save(&dat);

                    let s =
                        serde_json::to_string_pretty(&serde_json::from_slice::<Value>(&v).unwrap())
                            .unwrap();

                    fs::write(format!("{game}-{save}.json"), &s).unwrap();

                    // Parse JSON for structured comparison
                    if let (Ok(old_json), Ok(new_json)) = (
                        serde_json::from_str::<Value>(&previous),
                        serde_json::from_str::<Value>(&s),
                    ) {
                        let mut changes = Vec::new();
                        let mut log_changes = Vec::new();
                        let time = time::OffsetDateTime::now_local()
                            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
                        compare_json("", &old_json, &new_json, &mut changes, &mut log_changes);

                        if !changes.is_empty() {
                            change_count += 1;
                            println!("[{time}] Change #{change_count} detected:");
                            writeln!(&mut log_file, "[{time}] Change #{change_count} detected:")?;

                            for change in changes {
                                println!("  {change}");
                            }

                            for log_change in &log_changes {
                                writeln!(&mut log_file, "  {log_change}")?;
                            }

                            println!();
                            writeln!(&mut log_file)?;
                        }
                    }

                    previous.clone_from(&s);
                    last_modified = current_modified;
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    println!("File doesn't exist (yet)");
                    last_modified = None;
                }
                Err(e) => return Err(e),
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn decrypt_save(dat: &[u8]) -> Vec<u8> {
    let dat = &dat[CSHARP_HEADER.len()..dat.len() - 1];
    let (_length, nbytes) = ULEB128::read_from(dat).unwrap();
    let dat = &dat[nbytes..];

    let dat = BASE64_STANDARD.decode(dat).unwrap();

    ecb::Decryptor::<Aes256>::new(KEY.into())
        .decrypt_padded_vec_mut::<Pkcs7>(&dat)
        .unwrap()
}

fn encrypt_save(dat: &[u8]) -> Vec<u8> {
    let dat = ecb::Encryptor::<Aes256>::new(KEY.into()).encrypt_padded_vec_mut::<Pkcs7>(dat);

    let mut dat = BASE64_STANDARD.encode(&dat).into_bytes();

    let length = dat.len() as u64;
    dbg!(&dat.len());
    let mut buf = [0; 4];
    let nbytes = ULEB128::from(length).write_into(&mut buf).unwrap();
    dat.splice(0..0, buf[..nbytes].iter().copied());

    dat.splice(0..0, CSHARP_HEADER);
    dat.push(0x0B);

    dat
}
