use std::fs;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::thread;
use qrcode::{Color, QrCode};
use tiny_http::{Header, Method, Response, Server, StatusCode};

use crate::services::media::{MediaService, PlaybackStatus};
use crate::services::system_controls::SystemControlsService;
use crate::services::telemetry::TelemetryService;

#[derive(Debug, Clone)]
pub enum DeckCommand {
    TogglePlayPause,
    NextTrack,
    PreviousTrack,
    Seek(f32),
    SetVolume(f32),
    ToggleMute,
    ToggleMicMute,
}

#[derive(Debug, Default, Clone)]
pub struct DeckSharedState {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub position: f32,
    pub duration: f32,
    pub volume: f32,
    pub muted: bool,
    pub mic_muted: bool,
    pub cpu: f32,
    pub ram: f32,
    pub fps: f32,
    pub command_queue: Vec<DeckCommand>,
}

pub struct DeckServerService {
    pub server_url: String,
    pub qr_matrix: Vec<Vec<bool>>,
    pub port: u16,
    pub connected_devices: Arc<RwLock<usize>>,
    pub shared_state: Arc<RwLock<DeckSharedState>>,
}

impl Default for DeckServerService {
    fn default() -> Self {
        Self::new()
    }
}

impl DeckServerService {
    pub fn new() -> Self {
        let connected_devices = Arc::new(RwLock::new(0));
        let shared_state = Arc::new(RwLock::new(DeckSharedState::default()));

        let local_ip = detect_local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let mut port = 8088;

        let server = match Server::http(format!("0.0.0.0:{}", port)) {
            Ok(s) => s,
            Err(_) => {
                port = 8090;
                Server::http(format!("0.0.0.0:{}", port)).unwrap_or_else(|_| {
                    Server::http("0.0.0.0:0").expect("Failed to bind Deck HTTP server")
                })
            }
        };

        let actual_port = server.server_addr().to_ip().map(|a| a.port()).unwrap_or(port);
        let server_url = format!("http://{}:{}/", local_ip, actual_port);
        let qr_matrix = generate_qr(&server_url);

        let srv_state = Arc::clone(&shared_state);
        let srv_devices = Arc::clone(&connected_devices);

        thread::Builder::new()
            .name("deck_http_server".to_string())
            .spawn(move || {
                run_deck_server(server, srv_state, srv_devices);
            })
            .expect("Failed to spawn deck HTTP server thread");

        Self {
            server_url,
            qr_matrix,
            port: actual_port,
            connected_devices,
            shared_state,
        }
    }

    pub fn sync_and_poll_commands(
        &self,
        media: &mut MediaService,
        telemetry: &TelemetryService,
        controls: &mut SystemControlsService,
    ) {
        let mut lock = self.shared_state.write().unwrap();

        // 1. Update shared state for incoming HTTP poll requests
        let tr = media.current_track();
        lock.title = tr.title.clone();
        lock.artist = tr.artist.clone();
        lock.playing = media.status == PlaybackStatus::Playing;
        lock.position = tr.current_pos_secs;
        lock.duration = tr.duration_secs;

        lock.volume = controls.volume;
        lock.muted = controls.is_muted;
        lock.mic_muted = controls.is_mic_muted;

        lock.cpu = telemetry.cpu_usage;
        lock.ram = telemetry.ram_usage_pct;
        lock.fps = telemetry.fps;

        // 2. Drain incoming phone commands and apply them
        let commands: Vec<DeckCommand> = lock.command_queue.drain(..).collect();
        drop(lock);

        for cmd in commands {
            match cmd {
                DeckCommand::TogglePlayPause => media.toggle_play_pause(),
                DeckCommand::NextTrack => media.next(),
                DeckCommand::PreviousTrack => media.previous(),
                DeckCommand::Seek(pos) => media.seek_ratio(pos),
                DeckCommand::SetVolume(lvl) => controls.set_volume(lvl),
                DeckCommand::ToggleMute => controls.toggle_mute(),
                DeckCommand::ToggleMicMute => controls.toggle_mic_mute(),
            }
        }
    }
}

fn detect_local_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip())
}

fn generate_qr(url: &str) -> Vec<Vec<bool>> {
    let code = match QrCode::new(url.as_bytes()) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let width = code.width();
    let colors = code.to_colors();
    let mut matrix = Vec::with_capacity(width);

    for y in 0..width {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let is_dark = colors[y * width + x] == Color::Dark;
            row.push(is_dark);
        }
        matrix.push(row);
    }
    matrix
}

fn run_deck_server(
    server: Server,
    shared_state: Arc<RwLock<DeckSharedState>>,
    devices: Arc<RwLock<usize>>,
) {
    let assets_dir = PathBuf::from("assets/deck");

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().clone();

        // 1. API: Pair
        if url == "/api/pair" && method == Method::Post {
            *devices.write().unwrap() += 1;
            let json = r#"{"success":true,"data":{"ticket":"pair_1001","token":"notch-auth-token-1001","pc":"NotchIsland PC"}}"#;
            respond_json(request, json);
            continue;
        }

        // 2. API: Pair Poll
        if url == "/api/pair/poll" && method == Method::Post {
            let json = r#"{"success":true,"data":{"token":"notch-auth-token-1001"}}"#;
            respond_json(request, json);
            continue;
        }

        // 3. API: Deck layout and actions
        if url == "/api/deck" {
            let json = r#"{"success":true,"data":{"pc":"NotchIsland PC","actions":[
                {"id":"app:spotify","label":"Spotify","icon":"music"},
                {"id":"app:terminal","label":"Terminal","icon":"terminal"},
                {"id":"app:browser","label":"Browser","icon":"globe"},
                {"id":"game:shot","label":"Screenshot","icon":"camera"},
                {"id":"sys:lock","label":"Lock PC","icon":"lock"}
            ],"choices":[]}}"#;
            respond_json(request, json);
            continue;
        }

        // 4. API: Status poll (media, volume, meters)
        if url == "/api/status" {
            let state = shared_state.read().unwrap().clone();
            let json = format!(
                r#"{{"success":true,"data":{{"title":"{}","artist":"{}","playing":{},"position":{:.1},"duration":{:.1},"volume":{:.2},"muted":{},"micMuted":{},"cpu":{:.1},"ram":{:.1},"fps":{:.0}}}}}"#,
                state.title, state.artist, state.playing, state.position, state.duration,
                state.volume, state.muted, state.mic_muted, state.cpu, state.ram, state.fps
            );
            respond_json(request, &json);
            continue;
        }

        // 5. API: Volume adjustment
        if url == "/api/volume" && method == Method::Post {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(level) = val.get("level").and_then(|v| v.as_f64()) {
                    shared_state.write().unwrap().command_queue.push(DeckCommand::SetVolume(level as f32));
                }
            }
            respond_json(request, r#"{"success":true}"#);
            continue;
        }

        // 6. API: Seek playback
        if url == "/api/seek" && method == Method::Post {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(pos) = val.get("position").and_then(|v| v.as_f64()) {
                    shared_state.write().unwrap().command_queue.push(DeckCommand::Seek(pos as f32));
                }
            }
            respond_json(request, r#"{"success":true}"#);
            continue;
        }

        // 7. API: Action handler
        if url == "/api/action" && method == Method::Post {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(act_id) = val.get("id").and_then(|v| v.as_str()) {
                    let cmd = match act_id {
                        "play" | "pause" | "toggle" => Some(DeckCommand::TogglePlayPause),
                        "next" => Some(DeckCommand::NextTrack),
                        "prev" => Some(DeckCommand::PreviousTrack),
                        "mute" => Some(DeckCommand::ToggleMute),
                        "mic_mute" => Some(DeckCommand::ToggleMicMute),
                        _ => None,
                    };
                    if let Some(c) = cmd {
                        shared_state.write().unwrap().command_queue.push(c);
                    }
                }
            }
            respond_json(request, r#"{"success":true}"#);
            continue;
        }

        // 8. Static Files serving from assets/deck
        let file_path = if url == "/" || url == "/index.html" {
            assets_dir.join("index.html")
        } else {
            let clean_path = url.trim_start_matches('/');
            assets_dir.join(clean_path)
        };

        if file_path.exists() && file_path.is_file() {
            serve_file(request, &file_path);
        } else {
            let resp = Response::from_string("Not Found").with_status_code(StatusCode(404));
            let _ = request.respond(resp);
        }
    }
}

fn respond_json(request: tiny_http::Request, json: &str) {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    let cors = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap();
    let mut response = Response::from_string(json);
    response.add_header(header);
    response.add_header(cors);
    let _ = request.respond(response);
}

fn serve_file(request: tiny_http::Request, path: &Path) {
    let mime = match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "json" => "application/json",
        _ => "application/octet-stream",
    };

    if let Ok(bytes) = fs::read(path) {
        let content_type = Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap();
        let mut response = Response::from_data(bytes);
        response.add_header(content_type);
        let _ = request.respond(response);
    } else {
        let resp = Response::from_string("Error reading file").with_status_code(StatusCode(500));
        let _ = request.respond(resp);
    }
}
