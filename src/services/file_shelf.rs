use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::thread;
use qrcode::{Color, QrCode};
use tiny_http::{Header, Response, Server, StatusCode};

#[derive(Debug, Clone)]
pub struct ShelfItem {
    pub id: usize,
    pub name: String,
    pub size_bytes: u64,
    pub data: Vec<u8>,
    pub added_at: String,
}

impl ShelfItem {
    pub fn formatted_size(&self) -> String {
        let bytes = self.size_bytes;
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

pub struct FileShelfService {
    items: Arc<RwLock<Vec<ShelfItem>>>,
    next_id: Arc<RwLock<usize>>,
    pub server_url: String,
    pub qr_matrix: Vec<Vec<bool>>,
    pub port: u16,
}

impl Default for FileShelfService {
    fn default() -> Self {
        Self::new()
    }
}

impl FileShelfService {
    pub fn new() -> Self {
        let items = Arc::new(RwLock::new(Vec::new()));
        let next_id = Arc::new(RwLock::new(1));

        // Add sample starter items on the shelf
        {
            let mut list = items.write().unwrap();
            list.push(ShelfItem {
                id: 1,
                name: "Architecture_Spec.pdf".to_string(),
                size_bytes: 1024 * 342,
                data: b"%PDF-1.4 Mock Dynamic Island Native Architecture Blueprint".to_vec(),
                added_at: "Just now".to_string(),
            });
            list.push(ShelfItem {
                id: 2,
                name: "Screenshot_2026.png".to_string(),
                size_bytes: 1024 * 890,
                data: b"\x89PNG\r\n\x1a\nMock screenshot bytes".to_vec(),
                added_at: "2m ago".to_string(),
            });
            *next_id.write().unwrap() = 3;
        }

        // Determine best local IP address
        let local_ip = detect_local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        // Try ports starting from 8088
        let mut chosen_port = 8088;
        let server = match Server::http(format!("0.0.0.0:{}", chosen_port)) {
            Ok(s) => s,
            Err(_) => {
                chosen_port = 8089;
                Server::http(format!("0.0.0.0:{}", chosen_port)).unwrap_or_else(|_| {
                    Server::http("0.0.0.0:0").expect("Failed to bind any HTTP port")
                })
            }
        };

        let actual_port = server.server_addr().to_ip().map(|a| a.port()).unwrap_or(chosen_port);
        let server_url = format!("http://{}:{}/", local_ip, actual_port);

        // Generate QR code matrix for mobile scanning
        let qr_matrix = generate_qr(&server_url);

        // Spawn background HTTP server thread
        let server_items = Arc::clone(&items);
        let server_next_id = Arc::clone(&next_id);

        thread::Builder::new()
            .name("file_shelf_http".to_string())
            .spawn(move || {
                run_http_server(server, server_items, server_next_id);
            })
            .expect("Failed to spawn HTTP server thread");

        Self {
            items,
            next_id,
            server_url,
            qr_matrix,
            port: actual_port,
        }
    }

    pub fn get_items(&self) -> Vec<ShelfItem> {
        self.items.read().unwrap().clone()
    }

    pub fn add_file(&self, name: &str, data: Vec<u8>) {
        let mut id_lock = self.next_id.write().unwrap();
        let id = *id_lock;
        *id_lock += 1;

        let size_bytes = data.len() as u64;
        let mut list = self.items.write().unwrap();
        list.push(ShelfItem {
            id,
            name: name.to_string(),
            size_bytes,
            data,
            added_at: "Just now".to_string(),
        });
    }

    pub fn remove_file(&self, id: usize) {
        let mut list = self.items.write().unwrap();
        list.retain(|item| item.id != id);
    }

    pub fn clear(&self) {
        let mut list = self.items.write().unwrap();
        list.clear();
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

fn run_http_server(
    server: Server,
    items: Arc<RwLock<Vec<ShelfItem>>>,
    _next_id: Arc<RwLock<usize>>,
) {
    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if url == "/" || url == "/index.html" {
            // Serve clean, mobile-optimized file shelf page
            let current_items = items.read().unwrap().clone();
            let mut list_html = String::new();

            for item in &current_items {
                list_html.push_str(&format!(
                    r#"<div class="card">
                        <div class="file-info">
                            <span class="file-name">{}</span>
                            <span class="file-meta">{} &bull; {}</span>
                        </div>
                        <a href="/download/{}" class="btn">Download</a>
                    </div>"#,
                    item.name,
                    item.formatted_size(),
                    item.added_at,
                    item.id
                ));
            }

            if current_items.is_empty() {
                list_html = r#"<div class="card"><span class="file-meta">No files currently staged on the shelf. Drag files to the Notch on your PC!</span></div>"#.to_string();
            }

            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>NotchIsland &bull; Wi-Fi File Shelf</title>
    <style>
        :root {{
            --bg: #0d0f12;
            --card-bg: #1a1d24;
            --accent: #38bdf8;
            --text: #f1f5f9;
            --subtext: #94a3b8;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 24px 16px;
            display: flex;
            flex-direction: column;
            align-items: center;
        }}
        .header {{
            text-align: center;
            margin-bottom: 24px;
        }}
        .pill {{
            background: #000;
            border: 1px solid rgba(255, 255, 255, 0.15);
            border-radius: 9999px;
            padding: 6px 16px;
            font-size: 13px;
            font-weight: 600;
            letter-spacing: 0.5px;
            display: inline-block;
            margin-bottom: 12px;
            color: var(--accent);
        }}
        h1 {{
            font-size: 24px;
            margin: 0 0 8px 0;
            font-weight: 700;
        }}
        p {{
            color: var(--subtext);
            margin: 0;
            font-size: 14px;
        }}
        .container {{
            width: 100%;
            max-width: 480px;
            display: flex;
            flex-direction: column;
            gap: 12px;
        }}
        .card {{
            background: var(--card-bg);
            border-radius: 16px;
            padding: 16px 20px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            border: 1px solid rgba(255, 255, 255, 0.06);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        }}
        .file-info {{
            display: flex;
            flex-direction: column;
            gap: 4px;
        }}
        .file-name {{
            font-weight: 600;
            font-size: 15px;
        }}
        .file-meta {{
            font-size: 12px;
            color: var(--subtext);
        }}
        .btn {{
            background: var(--accent);
            color: #000;
            text-decoration: none;
            padding: 8px 16px;
            border-radius: 9999px;
            font-weight: 600;
            font-size: 13px;
            transition: opacity 0.2s;
        }}
        .btn:hover {{
            opacity: 0.85;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="pill">&bull; NOTCHISLAND WIRELESS SHELF &bull;</div>
        <h1>PC Staging Shelf</h1>
        <p>Direct peer-to-peer download to your mobile device</p>
    </div>
    <div class="container">
        {}
    </div>
</body>
</html>"#,
                list_html
            );

            let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
            let mut response = Response::from_string(html);
            response.add_header(header);
            let _ = request.respond(response);
        } else if let Some(id_str) = url.strip_prefix("/download/") {
            if let Ok(id) = id_str.parse::<usize>() {
                let current_items = items.read().unwrap();
                if let Some(item) = current_items.iter().find(|i| i.id == id) {
                    let content_type = Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..]).unwrap();
                    let disposition = Header::from_bytes(
                        &b"Content-Disposition"[..],
                        format!("attachment; filename=\"{}\"", item.name).as_bytes(),
                    ).unwrap();

                    let mut response = Response::from_data(item.data.clone());
                    response.add_header(content_type);
                    response.add_header(disposition);
                    let _ = request.respond(response);
                    continue;
                }
            }

            let response = Response::from_string("File Not Found").with_status_code(StatusCode(404));
            let _ = request.respond(response);
        } else {
            let response = Response::from_string("Not Found").with_status_code(StatusCode(404));
            let _ = request.respond(response);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_shelf_operations() {
        let shelf = FileShelfService::new();
        let initial_count = shelf.get_items().len();
        assert!(initial_count >= 2);

        shelf.add_file("test_doc.txt", b"Hello from NotchIsland".to_vec());
        let items = shelf.get_items();
        assert_eq!(items.len(), initial_count + 1);

        let added = items.iter().find(|i| i.name == "test_doc.txt").unwrap();
        assert_eq!(added.size_bytes, 22);
        assert_eq!(added.formatted_size(), "22 B");

        shelf.remove_file(added.id);
        assert_eq!(shelf.get_items().len(), initial_count);

        assert!(!shelf.qr_matrix.is_empty());
    }
}
