use directories::ProjectDirs;

use std::{error::Error, fs::create_dir_all,fs::remove_dir_all};
use std::path::PathBuf;
use log::{error, info};


pub fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "dockerbeam", "beamEngine").map(|proj_dirs| {
        // This will resolve to:
        // Linux: /home/username/.config/dockerbeam
        // macOS: /Users/username/Library/Application Support/com.dockerbeam.dockerbeam
        // Windows: C:\Users\username\AppData\Roaming\dockerbeam\dockerbeam
        info!("Generated Configuration Folder Path - {}",proj_dirs.config_dir().to_path_buf().display());
        proj_dirs.config_dir().to_path_buf()
    })
    
}

pub fn clear_files(){
    let file_path = get_config_path().expect("Error getting config path").join("beamfiles/");
    info!("Deleting BeamFiles (docker image tar file) - {} ",file_path.display());
    remove_dir_all(file_path).expect("ERR : ");
}


fn make_folders() -> Result<(), Box<dyn std::error::Error>> {

    let config_path = get_config_path()
        .ok_or("Could not determine config directory")?;
    
    // Create main config directory
    create_dir_all(&config_path)?;
    
    // Create beamfiles subdirectory properly
    let beam_path = config_path.join("beamfiles");
    create_dir_all(&beam_path)?;


    Ok(())
}


pub fn load_or_create_config() {
    make_folders();
    // this part of code needs some refactoring because of sudden shift of aim,some code wasent properly refactored.

}

pub fn match_error<T> (r:Result<T,Box<dyn Error>>){
    match r{
        Err(e)=>error!("Err : {e}"),
        _=>{}
    }
}

