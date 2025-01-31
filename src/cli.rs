use dialoguer::{Input,theme::ColorfulTheme,FuzzySelect};
use indicatif::{ProgressBar, ProgressStyle,ProgressState};
use std::fmt::Write;
use crate::dockerHandler::get_images_list;


pub fn take_input()-> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter KEY here")
        .interact_text()
        .expect("Failed to read input")
}



//use flat map here?
pub async fn select_docker()->String{
    let list = get_images_list().await;


    let docker_imgs:Vec<String> = list
    .iter()
    .map(|f|f.repo_tags.iter().map(|x|x.split(":").next().expect("Err:").to_string()).collect())
    .collect();

    let x = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the Docker to Beam ")
        .default(0)
        .items(&docker_imgs)
        .interact()
        .unwrap();
    docker_imgs[x].clone()
}



pub fn download_status_mod(total_size:u64)->ProgressBar{
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {speed} [{eta}] ",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .with_key("speed", |state: &ProgressState, w: &mut dyn Write| {
            let speed = state.per_sec() / 1_000_000.0; // Convert to MB/s
            write!(w, "{:.2} MB/s", speed).unwrap()
        })
        .progress_chars("#>-"),
    );
    
    pb
}





