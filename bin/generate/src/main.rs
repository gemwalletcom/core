use primitives::Platform;

use std::{
    fs::{self, DirEntry},
    process::Command,
    vec,
};

static ANDROID_PACKAGE_PREFIX: &str = "com.wallet.core";
static LANGUAGE_SWIFT: &str = "swift";
static LANGUAGE_KOTLIN: &str = "kotlin";
static LANG_KOTLIN_ETX: &str = "kt";

fn main() {
    let mut folders = vec!["crates/primitives"];
    // Add all gem_* crates that have typeshare models
    let gem_crates = vec![
        "crates/gem_algorand",
        "crates/gem_aptos",
        "crates/gem_bitcoin",
        "crates/gem_cardano",
        "crates/gem_cosmos",
        "crates/gem_evm",
        "crates/gem_graphql",
        "crates/gem_near",
        "crates/gem_polkadot",
        "crates/gem_solana",
        "crates/gem_stellar",
        "crates/gem_sui",
        "crates/gem_ton",
        "crates/gem_tron",
        "crates/gem_xrp",
    ];
    folders.extend(gem_crates);

    let platform_str = std::env::args().nth(1).expect("no platform specified");
    let platform_directory_path = std::env::args().nth(2).expect("no path specified");

    let platform = Platform::new(platform_str.as_str()).unwrap();

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
        "referral.rs",
        "slippage.rs",
    ]
    .to_vec();
    ignored_files.append(&mut ignored_files_by_platform(platform.clone()));

    //ignored_files(platform);

    for folder in folders {
        let src_path = format!("{}/src", folder);

        let typeshare_dir_path = format!("{}/typeshare", src_path);
        // process typeshare/ directory if crate has typeshare directory
        if fs::metadata(&typeshare_dir_path).is_ok() {
            let paths = get_paths(folder, typeshare_dir_path);
            process_paths(paths, folder, &platform, &platform_directory_path, &ignored_files);
        } else {
            // For other crates like primitives, scan all files
            let paths = get_paths(folder, src_path);
            process_paths(paths, folder, &platform, &platform_directory_path, &ignored_files);
        }
    }
}

fn process_paths(paths: Vec<String>, _folder: &str, platform: &Platform, platform_directory_path: &str, ignored_files: &[&str]) {
    for path in paths {
        // Example path:
        // ./crates/primitives/src/utxo.rs
        let vec: Vec<&str> = path.split("/src/").collect();
        let first_parts: Vec<&str> = vec[0].split('/').collect();
        let raw_module_name = first_parts[1];

        let module_name = map_crate_module_name(raw_module_name);
        let directory_paths: Vec<&str> = vec[1].split('/').collect();
        let mut directory_paths_capitalized = directory_paths
            .iter()
            .filter(|x| !x.starts_with('.'))
            .map(|&x| str_capitlize(x))
            .collect::<Vec<_>>();

        let path: &str = &directory_paths_capitalized.pop().unwrap(); //.as_str();
                                                                      //FIX: Change extension for kotlin
        let ios_new_file_name = file_name(path, LANGUAGE_SWIFT);

        // prepend the chain name and Generated folder to the path if module name is different from raw module name
        let ios_new_path = if raw_module_name != module_name {
            let chain_name = get_chain_name_from_crate(raw_module_name);
            format!("{}/Generated/{}", chain_name, ios_new_file_name.as_str())
        } else {
            format!("{}/{}", directory_paths_capitalized.clone().join("/"), ios_new_file_name.as_str())
        };

        // For Kotlin, create the same structure as iOS but with lowercase chain names
        let kt_new_file_name = file_name(path, LANG_KOTLIN_ETX);
        let kt_new_path = if raw_module_name != module_name {
            let chain_name = get_chain_name_from_crate(raw_module_name).to_lowercase();
            format!("{}/generated/{}", chain_name, kt_new_file_name)
        } else {
            let directory_paths_lowercased: Vec<String> = directory_paths_capitalized.iter().map(|x| x.to_lowercase()).collect();
            format!("{}/{}", directory_paths_lowercased.join("/"), kt_new_file_name)
        };
        if ignored_files.contains(directory_paths.last().unwrap()) {
            continue;
        }
        //FIX: change input/output file for kotlin
        let input_path = format!("./{}/src/{}", vec[0], directory_paths.join("/"));

        let ios_output_path = output_path(Platform::IOS, platform_directory_path, str_capitlize(module_name).as_str(), ios_new_path);
        let android_output_path = output_path(Platform::Android, platform_directory_path, module_name.to_lowercase().as_str(), kt_new_path.clone());
        
        // Generate package name based on the path structure
        let android_package_name = if raw_module_name != module_name {
            let chain_name = get_chain_name_from_crate(raw_module_name).to_lowercase();
            format!("{}.{}.{}.generated", ANDROID_PACKAGE_PREFIX, module_name, chain_name)
        } else {
            let directory_paths_lowercased: Vec<String> = directory_paths_capitalized.iter().map(|x| x.to_lowercase()).collect();
            let directory_package = directory_paths_lowercased.join(".");
            format!(
                "{}.{}{}",
                ANDROID_PACKAGE_PREFIX,
                module_name,
                if directory_package.is_empty() {
                    String::new()
                } else {
                    format!(".{}", directory_package)
                }
            )
        };

        match platform {
            Platform::IOS => {
                // println!(
                //     "Generate file for iOS: {}, output: {}",
                //     input_path, ios_output_path
                // );
                generate_files(LANGUAGE_SWIFT, input_path.as_str(), ios_output_path.as_str(), "");
            }
            Platform::Android => {
                // println!(
                //     "Generate file for Android: {}, output: {}",
                //     input_path, android_output_path
                // );
                generate_files(
                    LANGUAGE_KOTLIN,
                    input_path.as_str(),
                    android_output_path.as_str(),
                    android_package_name.as_str(),
                );
            }
        }
    }
}

fn output_path(platform: Platform, directory: &str, module_name: &str, path: String) -> String {
    match platform {
        Platform::IOS => format!("{}/{}/Sources/{}", directory, module_name, path),
        Platform::Android => format!("{}/{}/{}", directory, module_name, path),
    }
}

fn file_name(name: &str, file_extension: &str) -> String {
    let split: Vec<&str> = name.split('.').collect();
    let new_split: Vec<&str> = split[0].split('_').collect();
    let new_name = new_split.iter().map(|&x| str_capitlize(x)).collect::<Vec<_>>().join("");
    format!("{}.{}", new_name, file_extension)
}

fn generate_files(language: &str, input_path: &str, output_path: &str, package_name: &str) {
    Command::new("typeshare")
        .arg(input_path)
        .arg(format!("--lang={}", language))
        .arg(format!("--output-file={}", output_path))
        .arg(format!("--java-package={}", package_name))
        .output()
        .unwrap();
}

fn get_paths(_folder: &str, path: String) -> Vec<String> {
    let paths = match fs::read_dir(&path) {
        Ok(paths) => paths,
        Err(_) => {
            eprintln!("Warning: Could not read directory: {}", path);
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
fn ignored_files_by_platform(platform: Platform) -> Vec<&'static str> {
    match platform {
        Platform::IOS => vec!["balance.rs"],
        Platform::Android => vec!["asset_data.rs", "balance.rs"],
    }
}

fn clear_path(path: DirEntry) -> String {
    format!("{}", path.path().display())
}

fn str_capitlize(s: &str) -> String {
    format!("{}{}", s[..1].to_string().to_uppercase(), &s[1..])
}

fn map_crate_module_name(crate_name: &str) -> &str {
    if crate_name.starts_with("gem_") {
        "blockchain"
    } else {
        crate_name
    }
}

fn get_chain_name_from_crate(crate_name: &str) -> String {
    let chain_name = crate_name.split("_").last().unwrap();
    match chain_name {
        "evm" => "Ethereum".to_string(),
        _ => str_capitlize(chain_name),
    }
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
