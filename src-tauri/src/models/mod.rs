use std::{collections::HashMap, net::Ipv4Addr, sync::Arc, time::Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use tokio::net::UdpSocket;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub alias: String,
    pub device_type: String,
    pub device_model: Option<String>,
    #[serde(skip)]
    pub ip: String,
    #[serde(skip)]
    pub port: u16,
    pub ip_ending: Option<String>,
}

impl PartialEq for DeviceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
    }
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            alias: "".into(),
            device_type: "".into(),
            device_model: None,
            ip: "".into(),
            port: 0,
            ip_ending: None,
        }
    }
}

// TODO: change all String to &str type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceResponse {
    #[serde(flatten)]
    pub device_info: DeviceInfo,
    pub announcement: bool,
    pub fingerprint: String,
}

impl From<DeviceInfo> for DeviceResponse {
    fn from(device: DeviceInfo) -> Self {
        Self {
            device_info: device,
            fingerprint: "".into(),
            announcement: false,
        }
    }
}

impl PartialEq for DeviceResponse {
    // https://www.reddit.com/r/rust/comments/t8d6wb/comment/hznabrt
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}

#[derive(Clone, Debug)]
pub struct LocalSendDevice {
    pub socket: Option<Arc<UdpSocket>>,
    pub this_device: DeviceResponse,
    pub devices: Vec<DeviceInfo>,
    pub interface_addr: Ipv4Addr,
    pub multicast_addr: Ipv4Addr,
    pub multicast_port: u16,
}

pub type ReceiveState = Arc<Mutex<AppState>>;
pub type Sender<T> = UnboundedSender<T>;
pub type Receiver<T> = UnboundedReceiver<T>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Image,
    Video,
    Pdf,
    Text,
    Other,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ReceiveStatus {
    // TODO: add status for cancelled
    Waiting,            // waiting for sender to send the files
    Receiving,          // in an ongoing session, receiving files
    Finished,           // all files received (end of session)
    FinishedWithErrors, // finished but some files could not be received (end of session)
}

#[derive(Clone, Debug)]
pub enum ClientMessage {
    Allow(Vec<String>),
    Decline,
}

#[derive(Clone, Debug)]
pub enum ServerMessage {
    SendRequest(SendRequest),
    SendFileRequest((String, usize)),
    CancelSession,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub id: String,
    pub size: usize, // bytes
    pub file_name: String,
    pub file_type: FileType,
    // pub token: String,   // TODO: use this to verify while receiving the file
    // preview_data: type? // nullable
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    alias: String,
    version: String,
    deviceModel: String,
    deviceType: String,
    fingerprint: String,
    port: u16,
    protocol: String,
    download: Option<bool>, // if the download API (5.2 and 5.3) is active (optional, default: false)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendRequest {
    #[serde(rename = "info")]
    pub device_info: DeviceInfo,
    pub files: HashMap<String, FileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendInfo {
    pub file_id: String,
    pub token: String,
}

#[derive(Clone)]
pub struct ReceiveSession {
    pub sender: DeviceInfo,
    pub files: HashMap<String, FileInfo>,
    pub file_status: HashMap<String, ReceiveStatus>,
    pub destination_directory: String,
    pub start_time: Instant,
    pub status: ReceiveStatus,
}

impl ReceiveSession {
    pub fn new(sender: DeviceInfo, destination_directory: String) -> Self {
        Self {
            sender,
            destination_directory,
            files: HashMap::new(),
            file_status: HashMap::new(),
            start_time: Instant::now(),
            status: ReceiveStatus::Waiting,
        }
    }
}

pub struct AppState {
    pub(crate) device: LocalSendDevice,
    pub(crate) server_tx: Sender<ServerMessage>,
    pub(crate) client_rx: Receiver<ClientMessage>,
    pub(crate) receive_session: Option<ReceiveSession>,
}
