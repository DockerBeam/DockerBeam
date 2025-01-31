// webrtc.rs
use std::error::Error;
use std::sync::Arc;
use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::ice_transport::ice_gathering_state::RTCIceGatheringState;
use tokio::time::{sleep, Duration,Instant};
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use tokio::sync::mpsc;
use crate::{cli, dockerHandler, io};
use tokio_tungstenite::WebSocketStream;
use tokio::fs::File;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use console::style;
use indicatif::ProgressBar;
use webrtc::api::setting_engine::SettingEngine;
use log::{error, info};
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use std::sync::atomic::{AtomicU64, Ordering};


pub type WriteStream = futures_util::stream::SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::Message>;

#[derive(Debug, PartialEq)]
pub enum WebRTCMessageType {
    SDPOffer,
    SDPAnswer,
    ICECandidate,
    Unknown,
}

// Function to determine the message type
pub fn getMsgType(input: &str) -> WebRTCMessageType {
    let trimmed = input.trim();

    // Check for ICE candidate: starts with "candidate:"
    if trimmed.starts_with("candidate:") {
        return WebRTCMessageType::ICECandidate;
    }

    // Check for SDP: mandatory fields
    if trimmed.contains("v=0") && trimmed.contains("o=") && trimmed.contains("s=") {
        // Check for SDP Offer based on common attributes
        if trimmed.contains("a=sendrecv") || trimmed.contains("m=audio") || trimmed.contains("m=video") {
            return WebRTCMessageType::SDPOffer;
        } else {
            return WebRTCMessageType::SDPAnswer;
        }
    }

    WebRTCMessageType::Unknown
}

pub async fn init_peer_connection() -> Result<Arc<RTCPeerConnection>, Box<dyn Error>> {
    let mut setting_engine = SettingEngine::default();

    // Set custom ICE timeouts
    setting_engine.set_ice_timeouts(
        Some(Duration::from_secs(10)), // Disconnected timeout
        Some(Duration::from_secs(100)), // Failed timeout
        Some(Duration::from_secs(5)),  // Keep-alive interval
    );

    let api = APIBuilder::new()
        .with_setting_engine(setting_engine)
        .build();

    let mut config = RTCConfiguration {
        ice_servers: vec![
            // STUN servers
            RTCIceServer {
                urls: vec![
                    "stun:stun.l.google.com:19302".to_owned(),
                    "stun:stun1.l.google.com:19302".to_owned(),
                    "stun:stun2.l.google.com:19302".to_owned(),
                    "stun:stun3.l.google.com:19302".to_owned(),
                    "stun:stun4.l.google.com:19302".to_owned(),
                    "stun:stun.nextcloud.com:443".to_owned(),
                    "stun:stun.relay.metered.ca:80".to_owned(),
                ],
                ..Default::default()
            },
            // TURN server
            RTCIceServer {
                urls: vec!["turn:numb.viagenie.ca".to_owned()],
                username: "webrtc@live.com".to_owned(),
                credential: "muazkh".to_owned(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    config.ice_candidate_pool_size = 5; 

    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    
    
    let pc_clone = Arc::clone(&peer_connection);
    pc_clone.on_peer_connection_state_change(Box::new(move |state| {
        match state {
            RTCPeerConnectionState::New => println!("State: New"),
            RTCPeerConnectionState::Connecting => println!("Connecting to peer"),
            RTCPeerConnectionState::Connected => println!("Connected to peer"),
            RTCPeerConnectionState::Disconnected => {println!("State: Disconnected");error!("Peer Disconnected.");std::process::exit(1)},
            RTCPeerConnectionState::Failed => {println!("State: Failed - Timeout or peer disconnected.");error!("Peer Disconected or Timed-out")},
            RTCPeerConnectionState::Closed => {io::clear_files();std::process::exit(0)},
            _=>panic!(" ")
        }
        Box::pin(async {})
    }));

    Ok(peer_connection)
}

pub async fn create_offer(pc: &Arc<RTCPeerConnection>) -> Result<(String,Arc<RTCDataChannel>), Box<dyn Error>> {
    let config = RTCDataChannelInit {
        ordered: Some(false),
        //max_retransmits: Some(0), disabled for now ......
        ..Default::default()
    };
    let dc: Arc<RTCDataChannel> = pc.create_data_channel("beamEngine", Some(config)).await?;
    let pc_c = Arc::clone(&pc);
    setup_data_channel(&pc_c,&dc).await;
    
    let offer = pc.create_offer(None).await?;
    
    pc.set_local_description(offer.clone()).await?;
    info!("Offer Created");
    Ok((offer.sdp,dc))
}

pub async fn create_answer(pc: &Arc<RTCPeerConnection>, offer_sdp: String) -> Result<String, Box<dyn Error>> {
    let offer = RTCSessionDescription::offer(offer_sdp)?;
    pc.set_remote_description(offer).await?;
    
    let answer = pc.create_answer(None).await?;
    
    pc.set_local_description(answer.clone()).await?;
    info!("Answer generated");
    
    Ok(answer.sdp)
}

pub async fn handle_answer(pc: &Arc<RTCPeerConnection>, answer_sdp: String) -> Result<(), Box<dyn Error>> {
    let answer = RTCSessionDescription::answer(answer_sdp)?;
    pc.set_remote_description(answer).await?;
    info!("Answer Handeled");
    Ok(())
}

// bad code - needs fixing , remove mutex and make the writer variable condiditions better and just basically make this code better
pub fn setup_ice_handling(pc: &Arc<RTCPeerConnection>) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel(500);
    let tx2 = tx.clone();
    let pc2 = Arc::clone(pc);
    pc.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
        let tx = tx.clone();

        Box::pin(async move {
            if let Some(candidate) = c {
                let candidate_str = candidate.to_json().expect("ERR :");


                if let Err(err) = tx.send(candidate_str.candidate).await {
                    eprintln!("Failed to send ICE candidate: {:?}", err);
                }
            }
        })
    }));
    tokio::spawn(async move{
        loop{
            if pc2.ice_gathering_state() == RTCIceGatheringState::Complete {
                tx2.send("ICE_GATHERING_COMPLETE".to_string()).await.expect("ERR :");
                break;
            }
        }
 
    });
        
    rx
}

pub async fn handle_ice_candidate(pc: &Arc<RTCPeerConnection>, candidate: String) -> Result<(), Box<dyn Error>> {
    info!("An ice was Handled");
    let ice_candidate = RTCIceCandidateInit {
        candidate,
        ..Default::default()
    };
    pc.add_ice_candidate(ice_candidate).await.expect("ICE ERR");
    Ok(())
}


async fn setup_data_channel(pc:&Arc<RTCPeerConnection>,dc: &Arc<RTCDataChannel>) {    
    const BUFFERED_AMOUNT_LOW_THRESHOLD: usize = 512 * 1024; // 512 KB
    dc.set_buffered_amount_low_threshold(BUFFERED_AMOUNT_LOW_THRESHOLD)
    .await;

    dc.on_open(Box::new(move || {
        println!("{}",style("Waiting for Transfer..\n").cyan());

        Box::pin(async move {
            

        })


    }));

    let chunks = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let docker_name = Arc::new(Mutex::new(None::<String>));
    let pb = Arc::new(Mutex::new(None::<ProgressBar>));
    let dc_clone = Arc::clone(&dc);
    let pc = Arc::clone(&pc);
    let expected_size = Arc::new(Mutex::new(None::<usize>));
    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(100); 
    let current_size = Arc::new(AtomicU64::new(0));

dc.on_message(Box::new({move |msg| {
    let chunks = Arc::clone(&chunks);
    let expected_size = Arc::clone(&expected_size);
    let docker_name = Arc::clone(&docker_name);
    let pb = Arc::clone(&pb);
    let dc_clone = Arc::clone(&dc_clone);
    let pc = Arc::clone(&pc);
    let current_size = Arc::clone(&current_size);

    
    Box::pin(async move {
        if msg.is_string {
            let text = String::from_utf8(msg.data.to_vec()).unwrap();

            if text.starts_with("FILE_NAME:") {
                if let Some(size) = text.split(':').nth(1){
                    *docker_name.lock().await = Some(size.to_owned());
                }
                return;
            }
            
            if text.starts_with("FILE_SIZE:") {
                if let Ok(size) = text.split(':').nth(1).unwrap_or("0").parse::<usize>() {
                    *expected_size.lock().await = Some(size);
                    *pb.lock().await = Some(cli::download_status_mod(size as u64));
                }
                return;
            }

            if text == "END_OF_TRANSFER" {

                
                let locked_chunks = chunks.lock().await;
                let expected = expected_size.lock().await;
                

                let total_size: usize = locked_chunks.iter().map(|chunk| chunk.len()).sum();
                let mut all_data = Vec::with_capacity(total_size);
                
                for chunk in locked_chunks.iter() {
                    all_data.extend_from_slice(chunk);
                }
                
                if let Some(expected_size) = *expected {
                    if all_data.len() != expected_size {
                        error!("Size mismatch! Expected: {}, Got: {}", 
                                expected_size, all_data.len());
                    }
                }
                println!("Transfer completed. Processing image...");
                let tar_path = io::get_config_path().unwrap();
                let tar_path = tar_path.join("beamfiles/recv.tar");
                info!("tar path -  {}",tar_path.display());
                let mut file = File::create(&tar_path).await.expect("Error creating file");
                file.write_all(&all_data).await.expect("ERR writing docker tar file");
                file.flush().await.expect("ERR flushing file");

                let _x = dockerHandler::load_image_from_tar(tar_path.to_owned().to_str().unwrap()).await;
                io::match_error(Ok(_x));
                match dc_clone.send_text("END_OF_TRANSFER | ").await{
                    Ok(_)=>{tokio::time::sleep(Duration::from_millis(500)).await;}
                    _=>{}
                };
                pc.close().await.expect("unable to close session : ");
                

                
                
                return;
            }
        } else {
            let data = msg.data.to_vec();
            let mut locked_chunks = chunks.lock().await;
            let pb_guard = pb.lock().await;
            current_size.fetch_add(data.len() as u64, Ordering::Relaxed);
            locked_chunks.push(data.clone());

            let pbg_2 = pb_guard.clone();

            if last_update.elapsed() >= update_interval {
                let total_size = expected_size.lock().await.unwrap();
                tokio::task::spawn_blocking(move || {
                    let pb = pbg_2.as_ref().unwrap();
                    let size = current_size.load(Ordering::Relaxed);
                    pb.set_position(size);
                    if size == total_size as u64 {
                        pb.finish_with_message("Download complete!");
                    }
                });

            }
        }
    })
}
}));

}


