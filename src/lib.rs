//! # nixtract
//! nixtract is a library and command line tool to extract information from nix derivations.
//! The main way to use nixtract is to call the `nixtract` function with a flake reference and optionally a system and attribute path.
//! Alternatively, the underlying functions can be used directly to extract information from nix derivations.
//! ## Example
//! ```no_run
//! use nixtract::{nixtract, NixtractConfig};

//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let flake_ref = "nixpkgs";
//!     let system = Some("x86_64-linux");
//!     let attribute_path = Some("haskellPackages.hello");
//!     let config = NixtractConfig::default();
//!
//!     let derivations = nixtract(flake_ref, system, attribute_path, config)?;
//!
//!     for derivation in derivations {
//!         println!("{:?}", derivation);
//!     }
//!
//!     Ok(())
//! }
//! ```
//! ## Command Line
//! nixtract can also be used as a command line tool. For example:
//! ```sh
//! nixtract --target-flake-ref nixpkgs --target-system x86_64-linux --target-attribute-path haskellPackages.hello
//! ```

use ::std::sync::{Arc, Mutex};
use rayon::prelude::*;
use std::sync::mpsc;

use error::Result;

mod nix;
pub use nix::*;

pub mod error;
pub mod message;

#[derive(Debug, Clone)]
pub struct ProcessingArgs<'a> {
    pub collected_paths: &'a Arc<Mutex<std::collections::HashSet<String>>>,
    pub flake_ref: &'a String,
    pub system: &'a Option<String>,
    pub attribute_path: String,
    pub offline: bool,
    pub include_nar_info: bool,
    pub runtime_only: bool,
    pub binary_caches: &'a Vec<String>,
    pub lib: &'a nix::lib::Lib,
    pub tx: mpsc::Sender<DerivationDescription>,
    /// Used by the worker threads to communicate their status back to the main thread.
    /// This can for instance be used to update the UI.
    /// main.rs uses this channel to update the indicatif status bard.
    pub message_tx: Option<mpsc::Sender<message::Message>>,
    pub timeout_duration: Option<std::time::Duration>, // NEW FIELD
}

fn send_message(
    message_tx: &Option<mpsc::Sender<message::Message>>,
    message: message::Message,
) -> Result<()> {
    if let Some(tx) = message_tx {
        Ok(tx.send(message)?)
    } else {
        Ok(())
    }
}

fn process(args: ProcessingArgs) -> Result<()> {
    log::debug!("process: Starting for derivation: {:?}", args.attribute_path);

    // Inform the calling thread that we are starting to process the derivation
    send_message(
        &args.message_tx,
        message::Message {
            status: message::Status::Started,
            id: rayon::current_thread_index().unwrap(),
            path: args.attribute_path.clone(),
        },
    )?;

    log::debug!("process: Describing derivation: {}", args.attribute_path);
    let description = nix::describe_derivation(&nix::DescribeDerivationArgs::from(args.clone()))?;
    log::debug!("process: Finished describing derivation: {}", args.attribute_path);

    // Abort if we have reached to bootstrap stage
    if description.name == "bootstrap-tools" || description.name.starts_with("bootstrap-stage") {
        log::debug!("process: Skipping bootstrap derivation: {}", description.name);
        return Ok(());
    }

    // Inform the calling thread that we have described the derivation
    send_message(
        &args.message_tx,
        message::Message {
            status: message::Status::Completed,
            id: rayon::current_thread_index().unwrap(),
            path: description.attribute_path.clone(),
        },
    )?;

    // Send the DerivationDescription to the main thread
    args.tx.send(description.clone())?;
    log::debug!("process: Sent description for {} to main thread.", description.attribute_path);

    // use par_iter to call process on all children of this derivation
    log::debug!("process: Processing {} build inputs for {}.", description.build_inputs.len(), description.attribute_path);
    description
        .build_inputs
        .into_par_iter()
        .map(|build_input| -> Result<()> {
            // check if the build_input has already be processed
            let done = {
                let mut collected_paths = args.collected_paths.lock().unwrap();
                match &build_input.output_path {
                    None => {
                        log::warn!(
                            "process: Found a derivation without an output_path: {:?}",
                            build_input
                        );
                        false
                    }
                    Some(output_path) => !collected_paths.insert(output_path.clone()),
                }
            };

            if done {
                log::debug!(
                    "process: Skipping already processed derivation: {}",
                    build_input.attribute_path.to_string()
                );

                // Inform calling thread that the derivation was skipped if
                // requested.
                send_message(
                    &args.message_tx,
                    message::Message {
                        status: message::Status::Skipped,
                        id: rayon::current_thread_index().unwrap(),
                        path: build_input.attribute_path,
                    },
                )?;

                return Ok(());
            }

            log::debug!("process: Recursively processing build input: {}", build_input.attribute_path);
            // Call process with the build_input
            process(ProcessingArgs {
                attribute_path: build_input.attribute_path,
                tx: args.tx.clone(),
                message_tx: args.message_tx.clone(),
                timeout_duration: args.timeout_duration, // Propagate timeout
                ..args
            })
        })
        .collect::<Result<Vec<()>>>()?;
    log::debug!("process: Finished processing build inputs for {}.", args.attribute_path);

    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct NixtractConfig {
    pub offline: bool,
    pub include_nar_info: bool,
    pub runtime_only: bool,
    pub binary_caches: Option<Vec<String>>,
    pub message_tx: Option<mpsc::Sender<message::Message>>,
    pub timeout_duration: Option<std::time::Duration>, // NEW FIELD
}

pub fn nixtract(
    flake_ref: impl Into<String>,
    system: Option<impl Into<String>>,
    attribute_path: Option<impl Into<String>>,
    config: NixtractConfig,
) -> Result<impl Iterator<Item = DerivationDescription>> {
    log::info!("nixtract: Starting process.");

    // Convert the arguments to the expected types
    let flake_ref = flake_ref.into();
    let system = system.map(Into::into);
    let attribute_path = attribute_path.map(Into::into);

    log::debug!("nixtract: Resolved flake_ref: {}", flake_ref);
    log::debug!("nixtract: Resolved system: {:?}", system);
    log::debug!("nixtract: Resolved attribute_path: {:?}", attribute_path);

    let binary_caches = match config.binary_caches {
        None => {
            log::debug!("nixtract: Getting substituters from Nix configuration.");
            nix::substituters::get_substituters(flake_ref.clone())?
        }
        Some(caches) => {
            log::debug!("nixtract: Using provided binary caches: {:?}", caches);
            caches
        }
    };

    // Writes the `lib.nix` file to the tempdir and stores its path
    let lib = nix::lib::Lib::new()?;
    log::debug!("nixtract: Nix library path: {:?}", lib.path);

    // Create a channel to communicate DerivationDescription to the main thread
    let (tx, rx) = mpsc::channel();

    log::info!(
        "nixtract: Starting attribute path discovery for flake_ref: {}, system: {}, attribute_path: {:?}",
        flake_ref,
        system
            .clone()
            .unwrap_or("builtins.currentSystem".to_owned()),
        attribute_path.clone().unwrap_or_default()
    );

    let collected_paths: Arc<Mutex<std::collections::HashSet<String>>> =
        Arc::new(Mutex::new(std::collections::HashSet::new()));

    // call find_attribute_paths to get the initial set of derivations
    let attribute_paths =
        nix::find_attribute_paths(&flake_ref, &system, &attribute_path, &config.offline, &lib)?;
    log::debug!("nixtract: Found {} initial attribute paths.", attribute_paths.len());

    // Combine all AttributePaths into a single Vec
    let mut derivations: Vec<FoundDrv> = Vec::new();
    for found_attribute_path in attribute_paths {
        log::debug!("nixtract: Extending derivations with {} found drvs from attribute path: {}", found_attribute_path.found_drvs.len(), found_attribute_path.attribute_path);
        derivations.extend(found_attribute_path.found_drvs);
    }
    log::debug!("nixtract: Total initial derivations to process: {}.", derivations.len());

    for found_drv in derivations.clone() {
        match found_drv.output_path {
            None => log::warn!("nixtract: Found a derivation without an output_path: {:?}", found_drv),
            Some(output_path) => {
                let mut collected_paths = collected_paths.lock().unwrap();
                collected_paths.insert(output_path);
                log::debug!("nixtract: Added {} to collected paths.", found_drv.attribute_path);
            }
        }
    }

    // Spawn a new rayon thread to call process on every foundDrv
    rayon::spawn(move || {
        log::debug!("nixtract: Spawning rayon threads to process derivations.");
        derivations.into_par_iter().for_each(|found_drv| {
            let processing_args = ProcessingArgs {
                collected_paths: &collected_paths,
                flake_ref: &flake_ref,
                system: &system,
                attribute_path: found_drv.attribute_path,
                offline: config.offline,
                runtime_only: config.runtime_only,
                include_nar_info: config.include_nar_info,
                binary_caches: &binary_caches,
                lib: &lib,
                tx: tx.clone(),
                message_tx: config.message_tx.clone(),
                timeout_duration: config.timeout_duration, // Propagate timeout
            };
            match process(processing_args) {
                Ok(_) => {}
                Err(e) => log::warn!("nixtract: Error processing derivation: {}", e),
            }
        });
        log::debug!("nixtract: All initial derivations processed by rayon.");
    });

    log::info!("nixtract: Returning iterator for results.");
    Ok(rx.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use std::fs;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_main_fixtures() -> Result<()> {
        init();

        // For every subdirectory in the tests/fixtures directory
        for entry in fs::read_dir("tests/fixtures").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path().canonicalize().unwrap();
            if path.is_dir() {
                let config = NixtractConfig {
                    runtime_only: false,
                    binary_caches: None,
                    offline: false,
                    include_nar_info: false,
                    message_tx: None,
                    timeout_duration: None, // Add this to test config
                };

                log::info!("Running test for {:?}", path);

                let test_name = path
                    .components()
                    .last()
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap();
                let flake_ref = path.to_str().unwrap();
                let system: Option<String> = None;
                let attribute_path: Option<String> = None;

                let mut descriptions = nixtract(flake_ref, system, attribute_path, config).unwrap();

                match test_name {
                    "flake-direct-buildInput" => {}
                    "flake-direct-nativeBuildInput" => {}
                    "flake-three-levels" => {}
                    "flake-trivial-rust" => {
                        assert!(descriptions.any(|d| {
                            d.src.is_some_and(|s| {
                                s.git_repo_url == "https://github.com/hello-lang/Rust.git"
                            })
                        }));
                    }
                    "flake-trivial" => {}
                    s => panic!("Unknown test: {}", s),
                }
            }
        }
        Ok(())
    }
}
