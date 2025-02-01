use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use std::error::Error;
use tokio_tungstenite::WebSocketStream;
use std::sync::Arc;
use tokio_rustls::rustls::{ClientConfig, RootCertStore, pki_types::ServerName};
use url::Url;
use rustls_pki_types::{CertificateDer, pem::PemObject};
use tokio_tungstenite::connect_async_tls_with_config;
use tokio_tungstenite::Connector;
use log::{debug, error, info};




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignalMsg {
    pub code: String,
    pub data: String,
    pub id: String,
}

pub type WriteStream = futures_util::stream::SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::Message>;
pub type ReadStream = futures_util::stream::SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>;


pub async fn connect_server() -> Result<(WriteStream, ReadStream), Box<dyn Error>> {
    let ip = include_str!("../server.dockerbeam");
    dotenv::dotenv().ok();
    rustls::crypto::ring::default_provider().install_default().expect(" Err : ");
    
    
    let url = Url::parse(&ip)?;


    // Extract the domain from the URL for TLS verification
    let domain = url.host_str()
        .ok_or("Invalid domain in URL")?
        .to_string();


    let server_name = ServerName::try_from(domain)
        .map_err(|_| "Invalid server name")?;

    let mut rct = RootCertStore::empty();
    // let root_ca = Asset::get("data/LE_root.pem").unwrap();
    let root_ca = include_bytes!("../data/LE_root.pem");
    // let pem = root_ca.data.as_ref();
    let pem = root_ca.as_ref();

    let certs = CertificateDer::from_pem_slice(pem)?;

    rct.add(certs)?;
    


    let tls_config = ClientConfig::builder()
        .with_root_certificates(rct)
        .with_no_client_auth();
    let arc_tls = Arc::new(tls_config.clone());
    let con = Connector::Rustls(arc_tls);
    let (ws_stream, _) = match connect_async_tls_with_config(ip,None,false, Some(con)).await {
        Ok(v)=>v,
        Err(_)=>{error!("Unable to connect to signalling Server . Please try again in some time");std::process::exit(1)}
    };
 
    let (write, read) = ws_stream.split();
    Ok((write, read))
}

pub async fn send_message(
    write: &mut WriteStream,
    code: &str,
    data: &str,
    id: &str
) -> Result<(), Box<dyn Error>> {
    let msg = SignalMsg {
        code: code.to_string(),
        data: data.to_string(),
        id:id.to_string()
    };
    
    let msg_str = serde_json::to_string(&msg)?;
    write.send(Message::Text(msg_str)).await?;
    Ok(())
}

pub async fn receive_message(read: &mut ReadStream) -> Result<SignalMsg, Box<dyn Error>> {
    match read.next().await {
        Some(Ok(msg)) => {
            match msg.to_text() {
                Ok(text) => {
                    info!("Recieved message -> \n{text:?}\n");
                    match serde_json::from_str(text) {
                        Ok(signal_msg) => {
                            Ok(signal_msg)
                        },
                        Err(e) => {
                            error!("JSON parse error: {:?}", e);
                            Err(Box::new(e))
                        },
                    }
                },
                Err(e) => {debug!("ERR 30 ");std::process::exit(1)},
            }
        },
        Some(Err(e)) => {debug!("ERR 31 - {e}");std::process::exit(1)},
        None => {error!("Unexpected end of websocket stream");std::process::exit(1);}
    }
}