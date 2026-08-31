use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::UNIX_EPOCH,
};

const AUDIO_IDS: &[&str] = &[
    "escape",
    "tab",
    "backspace",
    "enter",
    "space",
    "insert",
    "delete",
    "home",
    "end",
    "page_up",
    "page_down",
    "copy",
    "cut",
    "paste",
    "colon",
    "comma",
    "backslash",
    "slash",
    "exclamationmark",
    "at",
    "hash",
    "dollar",
    "percent",
    "caret",
    "ampersand",
    "asterisk",
    "open_paren",
    "close_paren",
    "underscore",
    "tilde",
    "less_than",
    "greater_than",
    "open_bracket",
    "close_bracket",
    "open_curly_bracket",
    "close_curly_bracket",
    "pipe",
    "questionmark",
    "semicolon",
    "quote",
    "backtick",
    "minus",
    "period",
    "plus",
    "equals",
    "shift",
    "control",
    "alt",
    "meta",
    "num0",
    "num1",
    "num2",
    "num3",
    "num4",
    "num5",
    "num6",
    "num7",
    "num8",
    "num9",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "f1",
    "f2",
    "f3",
    "f4",
    "f5",
    "f6",
    "f7",
    "f8",
    "f9",
    "f10",
    "f11",
    "f12",
    "f13",
    "f14",
    "f15",
    "f16",
    "f17",
    "f18",
    "f19",
    "f20",
    "f21",
    "f22",
    "f23",
    "f24",
    "f25",
    "f26",
    "f27",
    "f28",
    "f29",
    "f30",
    "f31",
    "f32",
    "f33",
    "f34",
    "f35",
    "arrow_up",
    "arrow_down",
    "arrow_left",
    "arrow_right",
    "browser_back",
];

const DEFAULT_PIPER_MODEL: &str = "models/en_US-lessac-medium.onnx";
const DEFAULT_PIPER_VOICE: &str = "en_US-lessac-medium";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=KEYBOARD_TTS_SOURCE");
    println!("cargo:rerun-if-env-changed=ESPEAK_NG");
    println!("cargo:rerun-if-env-changed=PIPER_BIN");
    println!("cargo:rerun-if-env-changed=PIPER_MODEL");
    println!("cargo:rerun-if-env-changed=KEYBOARD_AUDIO_DIR");
    println!("cargo:rerun-if-env-changed=PATH");
    for path in [
        "/opt/homebrew/bin/espeak-ng",
        "/usr/local/bin/espeak-ng",
        "/opt/homebrew/bin/piper",
        "/usr/local/bin/piper",
    ] {
        watch_existing_path(Path::new(path));
    }
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let audio_dir = out_dir.join("audio");
    fs::create_dir_all(&audio_dir).expect("create generated audio directory");

    let backend = select_backend();
    println!("cargo:rerun-if-env-changed=KEYBOARD_VOICE");
    let cache_dir = audio_cache_dir(&out_dir, &backend);
    fs::create_dir_all(&cache_dir).expect("create audio cache directory");

    for (index, id) in AUDIO_IDS.iter().enumerate() {
        let path = audio_dir.join(format!("{id}.wav"));
        let cached_path = cache_dir.join(format!("{id}.wav"));
        let phrase = phrase_for_id(id);
        if is_valid_wav(&cached_path) {
            eprintln!(
                "keyboard-voice: cache hit [{}/{}] with {} -> {} ({phrase})",
                index + 1,
                AUDIO_IDS.len(),
                backend.name(),
                cached_path.display()
            );
            fs::copy(&cached_path, &path)
                .unwrap_or_else(|error| panic!("failed to copy cached audio for {id}: {error}"));
            continue;
        }
        eprintln!(
            "keyboard-voice: generating audio [{}/{}] with {} -> {} ({phrase})",
            index + 1,
            AUDIO_IDS.len(),
            backend.name(),
            cached_path.display()
        );
        let temporary_path = cache_dir.join(format!("{id}.tmp.wav"));
        generate_audio(&backend, id, &phrase, &temporary_path)
            .unwrap_or_else(|error| panic!("failed to generate audio for {id}: {error}"));
        fs::rename(&temporary_path, &cached_path)
            .unwrap_or_else(|error| panic!("failed to cache audio for {id}: {error}"));
        fs::copy(&cached_path, &path)
            .unwrap_or_else(|error| panic!("failed to copy generated audio for {id}: {error}"));
    }
    eprintln!(
        "keyboard-voice: audio cache for {} files is {}",
        AUDIO_IDS.len(),
        cache_dir.display()
    );

    let mut generated_rs =
        String::from("pub fn audio_bytes(id: &str) -> Option<&'static [u8]> {\n    match id {\n");
    for id in AUDIO_IDS {
        let path = audio_dir.join(format!("{id}.wav"));
        generated_rs.push_str(&format!(
            "        \"{id}\" => Some(include_bytes!({:?})),\n",
            path.to_string_lossy()
        ));
    }
    generated_rs.push_str("        _ => None,\n    }\n}\n");
    fs::write(out_dir.join("audio_assets.rs"), generated_rs).expect("write generated audio index");
}

#[derive(Debug)]
enum AudioBackend {
    Piper {
        command: PiperCommand,
        model: PathBuf,
    },
    Espeak {
        executable: String,
        voice: String,
    },
    Assets {
        directory: PathBuf,
    },
    Silent,
}

#[derive(Debug)]
enum PiperCommand {
    Binary { executable: String },
    Python { executable: String },
}

impl AudioBackend {
    fn name(&self) -> &'static str {
        match self {
            Self::Piper { .. } => "Piper",
            Self::Espeak { .. } => "eSpeak-ng",
            Self::Assets { .. } => "assets",
            Self::Silent => "silent",
        }
    }
}

fn audio_cache_dir(out_dir: &Path, backend: &AudioBackend) -> PathBuf {
    let target_dir = out_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap_or(out_dir);
    target_dir
        .join("keyboard-voice-audio")
        .join(backend.cache_key())
}

impl AudioBackend {
    fn cache_key(&self) -> String {
        let mut hasher = DefaultHasher::new();
        match self {
            Self::Piper { command, model } => {
                "piper".hash(&mut hasher);
                hash_path_metadata(&mut hasher, model);
                hash_path_metadata(&mut hasher, &model_config_path(model));
                let (kind, executable) = match command {
                    PiperCommand::Binary { executable } => ("binary", executable),
                    PiperCommand::Python { executable } => ("python", executable),
                };
                kind.hash(&mut hasher);
                hash_path_metadata(&mut hasher, Path::new(executable));
            }
            Self::Espeak { executable, voice } => {
                "espeak-ng".hash(&mut hasher);
                voice.hash(&mut hasher);
                hash_path_metadata(&mut hasher, Path::new(executable));
            }
            Self::Assets { directory } => {
                "assets".hash(&mut hasher);
                directory.to_string_lossy().hash(&mut hasher);
                for id in AUDIO_IDS {
                    hash_path_metadata(&mut hasher, &directory.join(format!("{id}.wav")));
                }
            }
            Self::Silent => "silent".hash(&mut hasher),
        }
        format!("{:016x}", hasher.finish())
    }
}

fn hash_path_metadata(hasher: &mut DefaultHasher, path: &Path) {
    path.to_string_lossy().hash(hasher);
    if let Ok(metadata) = fs::metadata(path) {
        metadata.len().hash(hasher);
        if let Ok(modified) = metadata.modified() {
            modified
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
                .hash(hasher);
        }
    }
}

fn model_config_path(model: &Path) -> PathBuf {
    PathBuf::from(format!("{}.json", model.display()))
}

fn is_valid_wav(path: &Path) -> bool {
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    bytes.len() > 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

fn select_backend() -> AudioBackend {
    let requested = env::var("KEYBOARD_TTS_SOURCE").unwrap_or_else(|_| "piper".to_owned());
    match requested.trim().to_ascii_lowercase().as_str() {
        "piper" => match find_piper() {
            Ok(Some((command, model))) => {
                println!(
                    "cargo:warning=keyboard-voice: using Piper model {}",
                    model.display()
                );
                AudioBackend::Piper { command, model }
            }
            Ok(None) => {
                if let Some((executable, voice)) = find_espeak() {
                    println!(
                    "cargo:warning=keyboard-voice: Piper is unavailable (install piper-tts, or set PIPER_BIN and PIPER_MODEL); falling back to espeak-ng {voice:?}."
                );
                    AudioBackend::Espeak { executable, voice }
                } else {
                    println!(
                    "cargo:warning=keyboard-voice: Piper and espeak-ng are unavailable; using silent audio placeholders."
                );
                    AudioBackend::Silent
                }
            }
            Err(error) => panic!("{error}"),
        },
        "espeak-ng" | "espeak" => {
            if let Some((executable, voice)) = find_espeak() {
                AudioBackend::Espeak { executable, voice }
            } else {
                panic!(
                    "KEYBOARD_TTS_SOURCE=espeak-ng was requested, but espeak-ng was not found. Install it or set ESPEAK_NG."
                );
            }
        }
        "assets" | "wav" => {
            let directory = env::var_os("KEYBOARD_AUDIO_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("assets/audio"));
            watch_existing_path_or_parent(&directory);
            for id in AUDIO_IDS {
                let source = directory.join(format!("{id}.wav"));
                watch_existing_path_or_parent(&source);
                if !source.is_file() {
                    panic!(
                        "KEYBOARD_TTS_SOURCE=assets requires {}, but the file does not exist",
                        source.display()
                    );
                }
            }
            AudioBackend::Assets { directory }
        }
        "silent" | "none" => AudioBackend::Silent,
        other => {
            panic!("unknown KEYBOARD_TTS_SOURCE={other:?}; use piper, espeak-ng, assets, or silent")
        }
    }
}

fn watch_existing_path(path: &Path) {
    if path.exists() {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn watch_existing_path_or_parent(path: &Path) {
    if path.exists() {
        watch_existing_path(path);
    } else if let Some(parent) = path.parent() {
        watch_existing_path(parent);
    }
}

fn find_piper() -> Result<Option<(PiperCommand, PathBuf)>, String> {
    let model_was_explicit = env::var_os("PIPER_MODEL").is_some();
    let executables = env::var("PIPER_BIN")
        .map(|path| vec![path])
        .unwrap_or_else(|_| vec!["piper".to_owned(), "piper-tts".to_owned()]);
    let models = env::var("PIPER_MODEL")
        .map(|path| vec![PathBuf::from(path)])
        .unwrap_or_else(|_| vec![PathBuf::from(DEFAULT_PIPER_MODEL)]);
    for model in &models {
        watch_existing_path_or_parent(model);
        watch_existing_path_or_parent(&model_config_path(model));
    }

    let binary = executables
        .into_iter()
        .find(|executable| command_succeeds(executable, "--help"));
    let python_piper = find_python_piper();
    let model = &models[0];
    let config = model_config_path(model);
    if !model.is_file() || !config.is_file() {
        if binary.is_some() || python_piper.is_some() {
            let description = if model_was_explicit {
                format!(
                    "PIPER_MODEL points to {}, but the model or its .json config is missing.",
                    model.display()
                )
            } else {
                format!("the default Piper model {} is missing.", model.display())
            };
            let instructions = if model_was_explicit {
                format!(
                    "Place the model and matching .json config at {} and rerun cargo.",
                    model.display()
                )
            } else {
                format!(
                    "Download it manually, then rerun cargo:\n  python3 -m pip install piper-tts\n  mkdir -p models\n  python3 -m piper.download_voices --data-dir models {DEFAULT_PIPER_VOICE}\n\nExpected files:\n  {DEFAULT_PIPER_MODEL}\n  {DEFAULT_PIPER_MODEL}.json"
                )
            };
            return Err(format!(
                "keyboard-voice: Piper is installed, but {description}\n\n{instructions}"
            ));
        }
        return Ok(None);
    }

    if let Some(executable) = binary {
        return Ok(Some((PiperCommand::Binary { executable }, model.clone())));
    }
    if let Some(executable) = python_piper {
        return Ok(Some((PiperCommand::Python { executable }, model.clone())));
    }
    Ok(None)
}

fn find_python_piper() -> Option<String> {
    if env::var_os("PIPER_BIN").is_some() {
        return None;
    }
    ["python3", "python"]
        .into_iter()
        .find(|executable| command_succeeds_with_args(executable, &["-m", "piper", "--help"]))
        .map(str::to_owned)
}

fn find_espeak() -> Option<(String, String)> {
    let voice = env::var("KEYBOARD_VOICE").unwrap_or_else(|_| "en-us+f3".to_owned());
    if let Ok(path) = env::var("ESPEAK_NG") {
        if command_succeeds(&path, "--version") {
            return Some((path, voice));
        }
    }
    for candidate in [
        "espeak-ng",
        "/opt/homebrew/bin/espeak-ng",
        "/usr/local/bin/espeak-ng",
    ] {
        if command_succeeds(candidate, "--version") {
            return Some((candidate.to_owned(), voice));
        }
    }
    None
}

fn command_succeeds(executable: &str, argument: &str) -> bool {
    command_succeeds_with_args(executable, &[argument])
}

fn command_succeeds_with_args(executable: &str, arguments: &[&str]) -> bool {
    Command::new(executable)
        .args(arguments)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn generate_audio(
    backend: &AudioBackend,
    id: &str,
    phrase: &str,
    output: &Path,
) -> Result<(), String> {
    match backend {
        AudioBackend::Piper { command, model } => {
            let status = match command {
                PiperCommand::Binary { executable } => {
                    let mut child = Command::new(executable)
                        .args(["--model"])
                        .arg(model)
                        .args(["--output_file"])
                        .arg(output)
                        .stdin(Stdio::piped())
                        .spawn()
                        .map_err(|error| format!("could not start Piper: {error}"))?;
                    child
                        .stdin
                        .take()
                        .ok_or_else(|| "could not open Piper stdin".to_owned())?
                        .write_all(phrase.as_bytes())
                        .map_err(|error| format!("could not send text to Piper: {error}"))?;
                    child
                        .wait()
                        .map_err(|error| format!("could not wait for Piper: {error}"))?
                }
                PiperCommand::Python { executable } => {
                    let model_name = model
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| format!("invalid Piper model path {}", model.display()))?;
                    let data_dir = model
                        .parent()
                        .filter(|path| !path.as_os_str().is_empty())
                        .unwrap_or_else(|| Path::new("."));
                    Command::new(executable)
                        .args(["-m", "piper", "--data-dir"])
                        .arg(data_dir)
                        .args(["-m", model_name, "-f"])
                        .arg(output)
                        .args(["--", phrase])
                        .status()
                        .map_err(|error| format!("could not start Piper Python module: {error}"))?
                }
            };
            if status.success() {
                Ok(())
            } else {
                Err(format!("Piper exited with {status} while generating {id}"))
            }
        }
        AudioBackend::Espeak { executable, voice } => {
            let status = Command::new(executable)
                .args(["-w"])
                .arg(output)
                .args(["-v"])
                .arg(voice)
                .args(["-s", "150", "-p", "52", "-a", "165"])
                .arg(phrase)
                .status()
                .map_err(|error| format!("could not start espeak-ng: {error}"))?;
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "espeak-ng exited with {status} while generating {id}"
                ))
            }
        }
        AudioBackend::Assets { directory } => fs::copy(directory.join(format!("{id}.wav")), output)
            .map(|_| ())
            .map_err(|error| format!("could not copy source WAV: {error}")),
        AudioBackend::Silent => write_silence_wav(output)
            .map_err(|error| format!("could not write silent WAV: {error}")),
    }
}

fn phrase_for_id(id: &str) -> String {
    match id {
        "num0" => "Zero".into(),
        "num1" => "One".into(),
        "num2" => "Two".into(),
        "num3" => "Three".into(),
        "num4" => "Four".into(),
        "num5" => "Five".into(),
        "num6" => "Six".into(),
        "num7" => "Seven".into(),
        "num8" => "Eight".into(),
        "num9" => "Nine".into(),
        "f1" => "F one".into(),
        "f2" => "F two".into(),
        "f3" => "F three".into(),
        "f4" => "F four".into(),
        "f5" => "F five".into(),
        "f6" => "F six".into(),
        "f7" => "F seven".into(),
        "f8" => "F eight".into(),
        "f9" => "F nine".into(),
        "f10" => "F ten".into(),
        "f11" => "F eleven".into(),
        "f12" => "F twelve".into(),
        "f13" => "F thirteen".into(),
        "f14" => "F fourteen".into(),
        "f15" => "F fifteen".into(),
        "f16" => "F sixteen".into(),
        "f17" => "F seventeen".into(),
        "f18" => "F eighteen".into(),
        "f19" => "F nineteen".into(),
        "f20" => "F twenty".into(),
        "f21" => "F twenty one".into(),
        "f22" => "F twenty two".into(),
        "f23" => "F twenty three".into(),
        "f24" => "F twenty four".into(),
        "f25" => "F twenty five".into(),
        "f26" => "F twenty six".into(),
        "f27" => "F twenty seven".into(),
        "f28" => "F twenty eight".into(),
        "f29" => "F twenty nine".into(),
        "f30" => "F thirty".into(),
        "f31" => "F thirty one".into(),
        "f32" => "F thirty two".into(),
        "f33" => "F thirty three".into(),
        "f34" => "F thirty four".into(),
        "f35" => "F thirty five".into(),
        "arrow_up" => "Arrow up".into(),
        "arrow_down" => "Arrow down".into(),
        "arrow_left" => "Arrow left".into(),
        "arrow_right" => "Arrow right".into(),
        "open_bracket" => "Open bracket".into(),
        "close_bracket" => "Close bracket".into(),
        "exclamationmark" => "Exclamation mark".into(),
        "at" => "At sign".into(),
        "hash" => "Hash".into(),
        "dollar" => "Dollar sign".into(),
        "percent" => "Percent".into(),
        "caret" => "Caret".into(),
        "ampersand" => "Ampersand".into(),
        "asterisk" => "Asterisk".into(),
        "open_paren" => "Left parenthesis".into(),
        "close_paren" => "Right parenthesis".into(),
        "underscore" => "Underscore".into(),
        "tilde" => "Tilde".into(),
        "less_than" => "Less than".into(),
        "greater_than" => "Greater than".into(),
        "open_curly_bracket" => "Open curly bracket".into(),
        "close_curly_bracket" => "Close curly bracket".into(),
        "questionmark" => "Question mark".into(),
        "plus" => "Plus".into(),
        "equals" => "Equals".into(),
        "browser_back" => "Back".into(),
        "page_up" => "Page up".into(),
        "page_down" => "Page down".into(),
        "backspace" => "Backspace".into(),
        "escape" => "Escape".into(),
        "space" => "Space".into(),
        "enter" => "Enter".into(),
        "shift" => "Shift".into(),
        "control" => "Control".into(),
        "alt" => "Alt".into(),
        "meta" => "Command".into(),
        _ => id.replace('_', " "),
    }
}

fn write_silence_wav(path: &Path) -> std::io::Result<()> {
    let sample_rate = 16_000u32;
    let channels = 1u16;
    let bits_per_sample = 16u16;
    let data_size = 1_600u32;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let mut file = fs::File::create(path)?;
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_size).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    file.write_all(&vec![0u8; data_size as usize])?;
    Ok(())
}
