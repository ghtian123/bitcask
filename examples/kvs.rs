use bitcask::{new_engine,Engine};
use clap::{App, AppSettings, Arg, SubCommand};
use std::process::exit;

fn main() {
    let matchs = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::GlobalVersion)
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the value of a string key to a string")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::with_name("VALUE")
                        .help("The string value of the key")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the string value of a given string key")
                .arg(Arg::with_name("KEY").required(true).help("A string key")),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove a Key")
                .arg(Arg::with_name("KEY").required(true).help("A string key")),
        )
        .get_matches();

    let mut store = new_engine::<String>(Engine::BITCASK, Some("./testdata".into())).unwrap();

    // let mut store = new_engine::<String>(Engine::SLED, Some("./testdata".into())).unwrap();
    // let mut store  = KvStore::open::<String>("./".into()).unwrap();

    match matchs.subcommand() {
        Some(("get", sub_m)) => {
            // println!("{:?}", sub_m);
            let key = sub_m.value_of("KEY").unwrap();
            match store.get(key.to_string()){
                Ok(res)=>{
                    println!("get res==>{:?}",res);
                },
                Err(e)=>{
                    println!("get res==>{:?}",e);
                },

            }
            exit(1);
        }
        Some(("set", sub_m)) => {
            // println!("{:?}", sub_m);
            let key = sub_m.value_of("KEY").unwrap();
            let value = sub_m.value_of("VALUE").unwrap();
            match store.set(key.to_string(), value.to_string()){
                Ok(res)=>{
                    println!("set res==>{:?}",res);
                },
                Err(e)=>{
                    println!("set res==>{:?}",e);
                },
            }
            exit(1);
        }
        Some(("rm", sub_m)) => {
            // println!("{:?}", sub_m);
            let key = sub_m.value_of("KEY").unwrap();
            match store.remove(key.to_string()){
                Ok(res)=>{
                    println!("rm res==>{:?}",res);
                },
                Err(e)=>{
                    println!("rm res==> {:?}",e);
                },
            }
            exit(1);
        }
        _ => println!("unsupprot command!!"),
    }
}
