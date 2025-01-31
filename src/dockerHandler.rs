use bollard::Docker;
use futures_util::StreamExt;
use libp2p::bytes::Bytes;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use bollard::image::ListImagesOptions;
use bollard::models::ImageSummary;
use bollard::image::ImportImageOptions;
use bollard::errors::Error;
use log::{debug, error, info, warn};
use crate::io;


pub async fn export(image: &str) -> String{
    let docker = Docker::connect_with_defaults().expect("Error");
    let mut data = docker.export_image(image);
    let file_path = io::get_config_path().expect("Error getting config path").join("beamfiles/send.tar");
    
    let mut file = File::create(&file_path).await.expect("Error creating file");

    while let Some(chunk) = data.next().await {
        let chunk = chunk.expect("Error reading Docker image data");
        file.write_all(&chunk).await.expect("Error writing to file");
    }

    file.flush().await.expect("Error flushing file");
    
    file_path.to_str().expect("Error converting path to string").to_owned()
}


pub async fn get_images_list() ->Vec<ImageSummary>{
    let docker = Docker::connect_with_defaults().expect("ERR : ");
        let options = Some(ListImagesOptions::<String> {
        all: true,
        ..Default::default()
    });
    let list = match docker.list_images(options).await{
        Ok(x)=>x,
        Err(e) => {error!("Error Retrieving Docker images , Please check if docker is running or not.");std::process::exit(1)}
    };
    list
}

pub fn check_avail(){
    match Docker::connect_with_defaults(){
        Err(_e) =>{error!("Docker Engine Not running or is not installed !!\nPlease start engine and re-run this command\n");std::process::exit(1)}
        _=>{}
    }
}


pub async fn load_image_from_tar(tar_path: &str) -> Result<(), Error> {
    let docker = Docker::connect_with_defaults().expect("ERR : ");
    let file_content = tokio::fs::read(tar_path).await?;

    let mut stream = docker
        .import_image(
            ImportImageOptions {
                ..Default::default()
            },
            Bytes::from(file_content),
            None,
        );

    while let Some(response) = stream.next().await {
        match response {
            Ok(output) => println!("Image imported with ID: {output:?}"),
            Err(e) => eprintln!("Error importing image: {:?}", e),
        }
    }

    Ok(())
}