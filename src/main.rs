use std::env;
use env_logger::Env;
use log::{error, info};

mod cli;
mod clientbeam;
mod serverbeacon;
mod dockerHandler;
mod io;
mod connectionHandler;
mod webrtcControl;

/// Entry point for the CLI
#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    println!(r"
______           _            ______                      
|  _  \         | |           | ___ \                     
| | | |___   ___| | _____ _ __| |_/ / ___  __ _ _ __ ___  
| | | / _ \ / __| |/ / _ \ '__| ___ \/ _ \/ _` | '_ ` _ \ 
| |/ / (_) | (__|   <  __/ |  | |_/ /  __/ (_| | | | | | |
|___/ \___/ \___|_|\_\___|_|  \____/ \___|\__,_|_| |_| |_|

                                                    v1.0.0

");

    println!("LICENSE: MIT (https://opensource.org/license/mit/)\n");

    if args.len() < 2 {
        println!(r"
    USAGE:
        dockerbeam [COMMAND] [ARGS] [OPTIONS]

    COMMANDS:
        send <image>    Push Docker image to another peer
        get @<peer>     Pull Docker image from a peer

    OPTIONS:
        --verbose       Show detailed operation logs
        --verbose-max   Show all debug logs (for troubleshooting)

    EXAMPLES:
        dockerbeam send my-app:latest      Send a specific image
        dockerbeam send                    (Select image interactively)
        dockerbeam get @MTI3LjAuMC4x       Pull from peer 'MTI3LjAuMC4x'
        dockerbeam get                     Pull from peer

        ");
        return;
    }

    let mut log_level = "error";

    if args[args.len()-1]== "--verbose"{
        log_level = "info";
        args.pop();
    }

    else if args[args.len()-1]== "--verbose-max"{
        log_level = "debug";
        args.pop();
    }

    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();
    info!("  Verbose Logging Enabled");


    io::load_or_create_config();
    dockerHandler::check_avail();

    match args[1].as_str() {
        "send" | "push" => {
            let docker_name:String;
            if args.len() == 3 as usize{
                docker_name = args[2].to_owned();
            }else {
                docker_name = cli::select_docker().await;
            }
            match clientbeam::send_docker_image(docker_name).await{
                Err(e) => error!("{e:?}"),
                _=>{}
            }
        }
        "get" | "recieve" | "pull"=> {
            
            let mut user_hash = "NULL";
            if args.len() >= 3 && args[2].starts_with("@"){
                user_hash = args[2].trim_start_matches('@');    
            }
        
            match serverbeacon::receive_docker_image(user_hash.to_string()).await{
                Err(e) => error!("{e:?}"),
                _=>{} 
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Available commands: send, get");
        }
    }
}