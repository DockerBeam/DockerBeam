use console::style;
use futures::SinkExt;
use serde::{Serialize, Deserialize};
use std::error::Error;

use crate::cli;
use crate::dockerHandler;
use crate::io;
use crate::connectionHandler;
use crate::webrtcControl;
use crate::webrtcControl::WebRTCMessageType;
use tokio::time::Duration;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use async_std::sync::Arc;

use log::{debug, error, info};


use arboard::Clipboard;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignalMsg {
    pub code: String,
    pub data: String,
    pub id: String,
}


pub type WriteStream = futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, tokio_tungstenite::tungstenite::Message>;
pub type ReadStream = futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>;


// main
pub async fn send_docker_image(docker_name: String)-> Result<(),Box<dyn Error>> {    


    let (mut write, mut read) = connectionHandler::connect_server().await?;

    connectionHandler::send_message(&mut write,"server","null","null").await?;


    let mainid = connectionHandler::receive_message(&mut read).await?.id;
    println!("ID - {}",style(&mainid).cyan());
    let mut clipboard = Clipboard::new()?;
    match clipboard.set_text(mainid.clone()){
        Ok(_)=>{println!("{}",style("(ID has been copied to clipboard)").color256(245))},
        Err(e)=>{info!("Unable to copy ID to clipboard ({e})")}
    }





    let pc = webrtcControl::init_peer_connection().await?;
    let pc_c = Arc::clone(&pc);
    let pc_cc = Arc::clone(&pc);


    let mut dc:Arc<RTCDataChannel>;

    let chunk_size = 16*1024; 
    let max_buffered_amount = 1*1024 * 1024; 


    pc.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
        // Set up handlers for the DataChannel
        let dc_clone = Arc::clone(&dc);
        let dc_clone2 = Arc::clone(&dc);


        let doc_name = docker_name.clone();


        let (tx,mut rx)= tokio::sync::mpsc::channel::<()>(1);


        tokio::spawn ({async move{
            dc_clone2.on_buffered_amount_low(Box::new(move || {
                let more_can_be_sent = tx.clone();
        
                Box::pin(async move {
                    // Send a signal that more can be sent
                    more_can_be_sent.send(()).await;
                })
            }))
            .await;
        }});


        dc.on_open(Box::new(move || {
            Box::pin(async move {

                info!("Exporting Image ...");
                println!("{}",style("Preparing Docker Image for Transfer.").cyan());
                let datapath = dockerHandler::export(&doc_name).await;
                info!("Image Exported !");
                println!("{}",style("Initiating Transfer..\n").cyan());

        
                let file_content = match tokio::fs::read(&datapath).await {
                    Ok(content) => content,
                    Err(e) => {
                        error!("Failed to read tar file: {:?}", e);
                        return;
                    }
                };
        

                let name_message = format!("FILE_NAME:{}", doc_name.clone());
                if let Err(e) = dc_clone.send_text(&name_message).await {
                    error!("Failed to send file size: {:?}", e);
                    return;
                }
        
                // Send file size first
                let size_msg = format!("FILE_SIZE:{}", file_content.len());
                if let Err(e) = dc_clone.send_text(&size_msg).await {
                    error!("Failed to send file size: {:?}", e);
                    return;
                }

                let ts = file_content.len() as u64;
                let pb = cli::download_status_mod(ts);
                let mut current_pos = 0;



                for chunk in file_content.chunks(chunk_size) {

                    let chunk_bytes = libp2p::bytes::Bytes::copy_from_slice(chunk);
                    dc_clone.send(&chunk_bytes).await.expect("Err sending chunk : ");

                    let buffered_amount = dc_clone.buffered_amount().await;
                    
                    current_pos += chunk_bytes.len() as u64;
                    pb.set_position(current_pos);
                    if current_pos>=ts{
                        pb.finish_with_message("Transfer completed !");
                    }

                    if buffered_amount + chunk_bytes.len() > max_buffered_amount {
                        // Wait for the signal that more can be sent
                        let _ = rx.recv().await;
                    }

                }

                // Send end signal
                if let Err(e) = dc_clone.send_text("END_OF_TRANSFER").await {
                    error!("Failed to send end signal: {:?}", e);
                    return;
                }
                println!("Image sent {}",style("successfully").green());
        
            })
        }));
        let pc_c = Arc::clone(&pc_c);

        dc.on_message(Box::new(move |msg| {
            let pc_c = Arc::clone(&pc_c);

            Box::pin(async move {
                let received_message = String::from_utf8(msg.data.to_vec()).unwrap();
                info!("recieved - {}",received_message);
                if received_message.starts_with("END_OF_TRANSFER"){
                    let _x = pc_c.close().await;
                    io::match_error(Ok(_x));
                }
            })
        }));
    
        Box::pin(async {})
    }));




    loop {
        match connectionHandler::receive_message(&mut read).await {
            Ok(msg) => {
                match msg.code.as_str() {
                    "confirmation" => { // by default its confirming because another step for confirmation now seems useless , will have to remove this whole procedure soon
                        connectionHandler::send_message(&mut write, "server-accept", &msg.data.clone(), &msg.id.clone()).await?;
                    },
                    "forward"=>{
                        match webrtcControl::getMsgType(&msg.data){
                            WebRTCMessageType::SDPOffer=>{ 
                                let ans = webrtcControl::create_answer(&pc, msg.data.clone()).await?;
                                connectionHandler::send_message(&mut write, "forward", &ans, &mainid.clone()).await?;
                                let mut rx = webrtcControl::setup_ice_handling(&pc);
                                while let Some(candidate) = rx.recv().await {
                                    if candidate == "ICE_GATHERING_COMPLETE"{
                                        break;
                                    }
                        
                                    connectionHandler::send_message(&mut write, "forward", &candidate, &mainid).await?;
                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                }
                                if pc_cc.connection_state() == RTCPeerConnectionState::Connected{
                                    write.close().await;
                                    info!("Dropped Websocket Writer");
                                }
                                
                                
                            },
                            WebRTCMessageType::SDPAnswer=>{ //ignore this ig?
                                println!("got answer");
                            },
                            WebRTCMessageType::ICECandidate=>{
                                webrtcControl::handle_ice_candidate(&pc, msg.data).await?;

                            },
                            WebRTCMessageType::Unknown=>{
                                println!("unknown answer")
                            }

                        }
                    },
                    "err"=>{
                        error!("{}",msg.data);
                        write.close().await;
                        std::process::exit(1);
                    },
                    _ => println!("Received wrong error: {:?}", msg),
                }
            },
            Err(e) => {
                error!("Error receiving message - {:?}", e);
                break;
            }
        }
        

}

Ok(())
}



