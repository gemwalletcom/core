use primitives::Platform;

use std::{
    fs::{self, DirEntry},
    path::Path,
    process::Command,
    vec,
};

static ANDROID_PACKAGE_PREFIX: &str = "com.wallet.core";
static LANGUAGE_SWIFT: &str = "swift";
static LANGUAGE_KOTLIN: &str = "kotlin";
static LANG_KOTLIN_ETX: &str = "kt";
static LANGUAGE_TYPESCRIPT: &str = "typescript";
static LANG_TYPESCRIPT_EXT: &str = "ts";

#[derive(Clone, Copy)]
enum GeneratorType {
    Swift,
    Kotlin,
    TypeScript,
}

fn main() {
    let folders = vec!["crates/primitives"];

    let platform_str = std::env::args().nth(1).expect("no platform specified");
    let platform_directory_path = std::env::args().nth(2).expect("no path specified");

    let generator_type = match platform_str.as_str() {
        "web" => GeneratorType::TypeScript,
        "ios" => GeneratorType::Swift,
        "android" => GeneratorType::Kotlin,
        other => panic!("unsupported generator target: {other}"),
    };

    let mut ignored_files: Vec<&'static str> = [
        "lib.rs",
        "mod.rs",
        "client.rs",
        "model.rs",
        "address.rs",
        "address_formatter.rs",
        "big_int_hex.rs",
        "hash.rs",
        "pubkey.rs",
        "ethereum_address.rs",
        "keccak.rs",
        "number_formatter.rs",
        "mode.rs",
        "quote.rs",
        "slippage.rs",
    ]
    .to_vec();
    let mut platform_ignored = ignored_files_by_generator(&generator_type);
    ignored_files.append(&mut platform_ignored);

    for folder in folders {
        let src_path = format!("{folder}/src");
        let paths = get_paths(folder, src_path);
        process_paths(paths, folder, &generator_type, &platform_directory_path, &ignored_files);
    }
}

fn process_paths(paths: Vec<String>, _folder: &str, generator_type: &GeneratorType, platform_directory_path: &str, ignored_files: &[&str]) {
    for path in paths {
        // Example path:
        // ./crates/primitives/src/utxo.rs
        let vec: Vec<&str> = path.split("/src/").collect();
        if vec.len() < 2 {
            continue;
        }

        let first_parts: Vec<&str> = vec[0].split('/').collect();
        if first_parts.len() < 2 {
            continue;
        }

        let module_name = first_parts[1];

        let directory_paths: Vec<&str> = vec[1].split('/').collect();
        let mut directory_paths_capitalized = directory_paths
            .iter()
            .filter(|x| !x.starts_with('.'))
            .map(|&x| str_capitlize(x))
            .collect::<Vec<_>>();

        if directory_paths_capitalized.is_empty() {
            continue;
        }

        let file_path = directory_paths_capitalized.pop().unwrap();

        let file_name_original = directory_paths.last().unwrap_or(&"");
        let allow_mod_for_swap = matches!(generator_type, GeneratorType::TypeScript)
            && *file_name_original == "mod.rs"
            && directory_paths.len() >= 2
            && directory_paths[directory_paths.len() - 2] == "swap";

        if ignored_files.contains(file_name_original) && !allow_mod_for_swap {
            continue;
        }

        let input_path = format!("./{}/src/{}", vec[0], directory_paths.join("/"));

        match generator_type {
            GeneratorType::Swift => {
                let ios_new_file_name = file_name(&file_path, LANGUAGE_SWIFT);
                let ios_new_path = format!("{}/{}", directory_paths_capitalized.join("/"), ios_new_file_name);
                let ios_output_path = output_path(Platform::IOS, platform_directory_path, str_capitlize(module_name).as_str(), ios_new_path);
                generate_files(LANGUAGE_SWIFT, input_path.as_str(), ios_output_path.as_str(), None);
            }
            GeneratorType::Kotlin => {
                let kt_new_file_name = file_name(&file_path, LANG_KOTLIN_ETX);
                let directory_paths_lowercased: Vec<String> = directory_paths_capitalized.iter().map(|x| x.to_lowercase()).collect();
                let kt_new_path = format!("{}/{}", directory_paths_lowercased.join("/"), kt_new_file_name);
                let android_output_path = output_path(Platform::Android, platform_directory_path, module_name, kt_new_path.clone());
                let directory_package = directory_paths_lowercased.join(".");
                let android_package_name = format!(
                    "{}.{}{}",
                    ANDROID_PACKAGE_PREFIX,
                    module_name,
                    if directory_package.is_empty() {
                        String::new()
                    } else {
                        format!(".{directory_package}")
                    }
                );
                generate_files(
                    LANGUAGE_KOTLIN,
                    input_path.as_str(),
                    android_output_path.as_str(),
                    Some(android_package_name.as_str()),
                );
            }
            GeneratorType::TypeScript => {
                let ts_new_file_name = file_name(&file_path, LANG_TYPESCRIPT_EXT);
                let directory_paths_lowercased: Vec<String> = directory_paths_capitalized.iter().map(|x| x.to_lowercase()).collect();
                let ts_new_path = format!("{}/{}", directory_paths_lowercased.join("/"), ts_new_file_name);
                let web_output_path = output_path_web(platform_directory_path, module_name, ts_new_path);
                generate_files(LANGUAGE_TYPESCRIPT, input_path.as_str(), web_output_path.as_str(), None);
            }
        }
    }
}

fn output_path(platform: Platform, directory: &str, module_name: &str, path: String) -> String {
    match platform {
        Platform::IOS => format!("{directory}/{module_name}/Sources/{path}"),
        Platform::Android => format!("{directory}/{module_name}/{path}"),
    }
}

fn output_path_web(directory: &str, module_name: &str, path: String) -> String {
    format!("{directory}/{module_name}/{path}")
}

fn file_name(name: &str, file_extension: &str) -> String {
    let split: Vec<&str> = name.split('.').collect();
    let new_split: Vec<&str> = split[0].split('_').collect();
    let new_name = new_split.iter().map(|&x| str_capitlize(x)).collect::<Vec<_>>().join("");
    format!("{new_name}.{file_extension}")
}

fn generate_files(language: &str, input_path: &str, output_path: &str, package_name: Option<&str>) {
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let mut command = Command::new("typeshare");
    command
        .arg(input_path)
        .arg(format!("--lang={language}"))
        .arg(format!("--output-file={output_path}"));

    if let Some(package_name) = package_name {
        command.arg(format!("--java-package={package_name}"));
    }

    command.output().unwrap();
}

fn get_paths(_folder: &str, path: String) -> Vec<String> {
    let paths = match fs::read_dir(&path) {
        Ok(paths) => paths,
        Err(_) => {
            eprintln!("Warning: Could not read directory: {path}");
            return vec![];
        }
    };
    let mut result: Vec<String> = vec![];

    for path in paths {
        let dir_entry = path.unwrap();
        if dir_entry.path().is_dir() {
            let path_recursive = get_paths(_folder, clear_path(dir_entry));
            result.extend(path_recursive)
        } else {
            result.push(clear_path(dir_entry));
        }
    }

    result
}

//TODO: Pass from the command
fn ignored_files_by_generator(generator_type: &GeneratorType) -> Vec<&'static str> {
    match generator_type {
        GeneratorType::Swift => vec![""],
        GeneratorType::Kotlin => vec!["asset_data.rs"],
        GeneratorType::TypeScript => vec!["transaction_input_type.rs"],
    }
}

fn clear_path(path: DirEntry) -> String {
    format!("{}", path.path().display())
}

fn str_capitlize(s: &str) -> String {
    format!("{}{}", s[..1].to_string().to_uppercase(), &s[1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_name() {
        assert_eq!(file_name("token.rs", LANGUAGE_SWIFT), "Token.swift");
        assert_eq!(file_name("token_type.rs", LANGUAGE_SWIFT), "TokenType.swift");
    }

    #[test]
    fn test_str_capitlize() {
        assert_eq!(str_capitlize("balance"), "Balance");
        assert_eq!(str_capitlize("Balance"), "Balance");
    }
}
