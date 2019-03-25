use crate::store::Store;
use clap::{App, Arg, ArgMatches, SubCommand};
use prettytable::format;
use prettytable::Table;
use std::error;
use std::fs;
use std::iter::repeat;
use std::path::{Component, Path};

/// Create a simple table with no headers and aligned columns
fn simple_table() -> Table {
    let mut table = Table::new();
    let format = format::FormatBuilder::new().padding(0, 1).build();
    table.set_format(format);
    table
}

/// Add a new bucket to create directories in
fn handle_bucket_add(name: &str, dir: &str) -> Result<(), Box<dyn error::Error + 'static>> {
    let path = Path::new(dir);
    if !path.exists() {
        println!("Path does not exist: {}", dir);
        return Ok(());
    }

    match Store::load() {
        Ok(mut store) => {
            store.add_bucket(name, dir)?;
        }
        Err(e) => println!("{}", e),
    }
    Ok(())
}

/// Set the default bucket for creating new directories
fn handle_bucket_default(name: Option<&str>) -> Result<(), Box<dyn error::Error + 'static>> {
    match name {
        Some(name) => match Store::load() {
            Ok(mut store) => {
                store.set_default_bucket(name)?;
            }
            Err(e) => println!("{}", e),
        },
        None => match Store::load() {
            Ok(store) => {
                if let Some(bucket) = store.default_bucket() {
                    println!("{}", bucket.name);
                } else {
                    println!("Default bucket is not set");
                }
            }
            Err(e) => println!("{}", e),
        },
    }
    Ok(())
}

/// Forget a bucket that's being tracked
///
/// Doesn't remove the bucket contents, but forgets about it
///
/// If the bucket was the default bucket, the default bucket becomes
/// unset until manually changed.
///
/// # Example
///
/// ```
/// zz bucket forget my_bucket
/// ```
fn handle_bucket_forget(name: &str) -> Result<(), Box<dyn error::Error + 'static>> {
    match Store::load() {
        Ok(mut store) => {
            let original_length = store.buckets().len();
            store.forget_bucket(name)?;
            let new_length = store.buckets().len();
            if new_length == original_length {
                println!("Bucket '{}' does not exist", name);
            }
        }
        Err(e) => println!("{}", e),
    }
    Ok(())
}

/// List buckets by name with its path
///
/// # Example
///
/// ```
/// # List buckets
/// zz bucket ls
/// ```
fn handle_bucket_ls() {
    match Store::load() {
        Ok(store) => {
            let mut table = simple_table();
            for bucket in store.buckets().into_iter() {
                table.add_row(row![bucket.name, bucket.path]);
            }
            table.printstd();
        }
        Err(e) => println!("{}", e),
    }
}

/// List all directories across buckets
///
/// # Example
///
/// ```
/// # Long list
/// zz ls
///
/// # List directories in a specific bucket
/// zz ls -b my_bucket
/// ```
fn handle_ls(filter_bucket_name: Option<&str>) {
    match Store::load() {
        Err(e) => println!("{}", e),
        Ok(store) => {
            let mut table = simple_table();
            store
                .buckets()
                .into_iter()
                .filter(|b| match filter_bucket_name {
                    Some(bucket_name) => b.name == bucket_name,
                    None => true,
                })
                .filter_map(|b| match fs::read_dir(Path::new(&b.path)) {
                    Ok(result) => Some((b, result)),
                    Err(err) => {
                        println!("Unable to read dir: {}", err);
                        None
                    }
                })
                .flat_map(|(bucket, read_result)| repeat(bucket).zip(read_result))
                .for_each(|(bucket, dir)| match dir {
                    Ok(dir) => {
                        let path = dir.path();
                        let path_str = path.to_str().unwrap();
                        let last_part = path.components().last();
                        if let Some(Component::Normal(last)) = last_part {
                            let name_with_date = last.to_str().unwrap();
                            let strings: Vec<&str> = name_with_date.splitn(2, "_").collect();
                            let (date, name) = (strings[0], strings[1]);
                            table.add_row(row![bucket.name, date, name, path_str]);
                        } else {
                            panic!(format!("Couldn't get dir name from path: {}", path_str));
                        }
                    }
                    Err(err) => {
                        println!("Error reading dir: {}", err);
                    }
                });
            table.printstd();
        }
    }
}

/// Make a new directory in a bucket
///
/// By default this will create a new directory prefixed with the current date
/// in the default bucket.
///
/// # Errors
///
/// - When `-b/--bucket` is used, and the bucket doesn't exist
/// - When `-b/--bucket` is not used and there is no default bucket
fn handle_mkdir(name: &str, bucket: Option<&str>) {
    match Store::load() {
        Ok(store) => {
            let selected_bucket = match bucket {
                Some(bucket_name) => store.find_bucket(bucket_name),
                None => store.default_bucket(),
            };

            if let Some(bucket) = selected_bucket {
                match bucket.make_dir(name) {
                    Ok(path) => println!("{}", path.to_str().unwrap()),
                    Err(e) => println!("Error: {}", e),
                };
            } else {
                println!("No bucket to choose from");
            }
        }
        Err(e) => println!("{}", e),
    }
}

/// Parse CLI arguments
pub fn parse_args<'a>() -> Result<ArgMatches<'a>, Box<dyn error::Error + 'static>> {
    let matches = App::new("zz")
        .version("0.1.0")
        .author("Erik R. <eronshagen@gmail.com>")
        .about("Manage temporary working directories")
        .subcommand(
            SubCommand::with_name("bucket")
                .about("Manage buckets")
                .subcommand(
                    SubCommand::with_name("add")
                        .arg(
                            Arg::with_name("NAME")
                                .help("Name of the bucket")
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("DIR")
                                .help("Path to the bucket")
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("default")
                        .about("Get or set the default bucket")
                        .arg(
                            Arg::with_name("NAME")
                                .help("Set the default bucket to this bucket")
                                .required(false),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("forget").arg(
                        Arg::with_name("NAME")
                            .help("Name of the bucket")
                            .required(true),
                    ),
                )
                .subcommand(SubCommand::with_name("ls").about("List buckets")),
        )
        .subcommand(
            SubCommand::with_name("ls").about("List directories").arg(
                Arg::with_name("bucket")
                    .short("b")
                    .long("bucket")
                    .value_name("BUCKET_NAME")
                    // .takes_value(true) ???
                    .help("List directories in this bucket"),
            ),
        )
        .subcommand(
            SubCommand::with_name("mkdir")
                .about("Make a new directory")
                .arg(
                    Arg::with_name("bucket")
                        .help("Select bucket to create the directory in")
                        .short("b")
                        .long("bucket")
                        .value_name("BUCKET_NAME"),
                )
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the dir")
                        .required(true),
                ),
        )
        .get_matches();
    Ok(matches)
}

/// Dispatch sub-command handlers based on the parsed args
pub fn handle_parsed_args(matches: ArgMatches) -> Result<(), Box<dyn error::Error + 'static>> {
    if let Some(matches) = matches.subcommand_matches("bucket") {
        if let Some(matches) = matches.subcommand_matches("add") {
            handle_bucket_add(
                matches.value_of("NAME").unwrap(),
                matches.value_of("DIR").unwrap(),
            )?;
        } else if let Some(matches) = matches.subcommand_matches("default") {
            handle_bucket_default(matches.value_of("NAME"))?;
        } else if let Some(matches) = matches.subcommand_matches("forget") {
            handle_bucket_forget(matches.value_of("NAME").unwrap())?
        } else if let Some(_matches) = matches.subcommand_matches("ls") {
            handle_bucket_ls()
        }
    } else if let Some(matches) = matches.subcommand_matches("ls") {
        handle_ls(matches.value_of("bucket"));
    } else if let Some(matches) = matches.subcommand_matches("mkdir") {
        handle_mkdir(
            matches.value_of("NAME").unwrap(),
            matches.value_of("bucket"),
        );
    }
    Ok(())
}
