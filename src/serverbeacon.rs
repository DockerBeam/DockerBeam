use futures_util::{SinkExt, StreamExt};
use std::error::Error;

use crate::connectionHandler;
use crate::cli;
use crate::webrtcControl;
use crate::webrtcControl::WebRTCMessageType;
use tokio::time::Duration;
use webrtc::data_channel::RTCDataChannel;
use async_std::sync::Arc;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use log::info;
use log::error;


// need to do trickle ICE

pub async fn receive_docker_image(user_hash: String)->Result<(), Box<dyn Error>> {

    let pc = webrtcControl::init_peer_connection().await?;
    let pc_c = Arc::clone(&pc);
    let mut dc:Arc<RTCDataChannel>;
    

    let (mut write, mut read) = connectionHandler::connect_server().await?;
    let opp_id;
    if user_hash == "NULL"{
        opp_id = cli::take_input();
    }else{
        opp_id = user_hash.clone();
    };

    connectionHandler::send_message(&mut write,"client",&opp_id,&opp_id).await?;
    let mut my_id:String = "NULL".to_owned();

    loop {
        match connectionHandler::receive_message(&mut read).await {
            Ok(msg) => {
                match msg.code.as_str() {
                    "confirmation" => {
                        // this needs sorting
                        println!("confirmed");
                    },
                    "server-accepted" =>{
                        my_id = msg.data.clone();
                        let (offer,dcx) = webrtcControl::create_offer(&pc).await?;
                        dc = Arc::clone(&dcx);
                        connectionHandler::send_message(&mut write,"forward",&offer,&my_id).await?;
                    },
                    "forward"=>{
                        match webrtcControl::getMsgType(&msg.data){
                            WebRTCMessageType::SDPOffer=>{ //this is considered as answer since we are already generating offer above :)
                                webrtcControl::handle_answer(&pc, msg.data.clone()).await?;
                                // make it async
                                
                                let mut rx = webrtcControl::setup_ice_handling(&pc);
                                while let Some(candidate) = rx.recv().await {
                                    if candidate == "ICE_GATHERING_COMPLETE"{
                                        break;
                                    }
                                    connectionHandler::send_message(&mut write, "forward", &candidate, &my_id.clone()).await.expect("ERR : ");
                                    tokio::time::sleep(Duration::from_millis(100)).await;
                                }
                                if pc_c.connection_state() == RTCPeerConnectionState::Connected{
                                    write.close().await;
                                    info!("Dropped Websocket Writer");

                                }

                            },
                            WebRTCMessageType::SDPAnswer=>{
                                println!("got answer");
                            },
                            WebRTCMessageType::ICECandidate=>{
                                webrtcControl::handle_ice_candidate(&pc, msg.data).await?;
                            },
                            WebRTCMessageType::Unknown=>{
                                println!("Err: Bad word - {} ",msg.data);
                            }

                        }
                    },
                    "err"=>{
                        error!("{}",msg.data);
                    },
                    _ => println!("Received unknown command : {:?}", msg),
                }
            },
            Err(e) => {
                println!("Error receiving message: {:?}", e);
                break;
            }


        }



            

    }
    
    Ok(())
 
}