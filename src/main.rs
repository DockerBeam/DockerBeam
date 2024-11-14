use std::env;

mod serverbeacon;
mod clientbeam;

/// Entry point for the CLI
#[tokio::main]
async fn main() {
    // Collect the arguments from the command line
    let args: Vec<String> = env::args().collect();

    // Check if we have at least one command
    if args.len() < 2 {
        eprintln!("Usage: dockerbeam <command> [options]");
        return;
    }

    // Process the first argument as the command
    match args[1].as_str() {
        "send" => {
            if args.len() < 4 {
                eprintln!("Usage: dockerbeam send @<user_hash> <docker_name>");
                return;
            }
            
            // Validate user hash (should start with '@')
            let user_hash = &args[2];
            if !user_hash.starts_with('@') {
                eprintln!("Error: User hash should start with '@'");
                return;
            }
            let user_hash = user_hash.trim_start_matches('@'); 
            
            // Docker image name
            let docker_name = &args[3];

            // Call the clientBeam function to send the image
            clientbeam::send_docker_image(user_hash.to_string(), docker_name.to_string());
        }
        "get" || "recieve" => {
            // Call the serverBeacon function to receive the image
            serverbeacon::receive_docker_image().await;
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Available commands: send, get");
        }
    }
}