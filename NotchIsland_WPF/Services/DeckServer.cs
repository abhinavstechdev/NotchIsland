using System;
using System.IO;
using System.Net;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace NotchIsland.Services
{
    public class DeckServer
    {
        private HttpListener? _listener;
        private CancellationTokenSource? _cts;
        public int Port { get; } = 8088;
        public bool IsRunning { get; private set; }

        public void Start()
        {
            if (IsRunning) return;

            try
            {
                _listener = new HttpListener();
                _listener.Prefixes.Add($"http://localhost:{Port}/");
                _listener.Prefixes.Add($"http://127.0.0.1:{Port}/");
                _listener.Start();
                IsRunning = true;
                _cts = new CancellationTokenSource();

                Task.Run(() => ListenLoop(_cts.Token));
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"DeckServer start error: {ex.Message}");
            }
        }

        private async Task ListenLoop(CancellationToken token)
        {
            var baseDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Assets", "deck");

            while (!token.IsCancellationRequested && _listener != null && _listener.IsListening)
            {
                try
                {
                    var ctx = await _listener.GetContextAsync();
                    _ = Task.Run(() => HandleRequest(ctx, baseDir));
                }
                catch
                {
                    if (token.IsCancellationRequested) break;
                }
            }
        }

        private void HandleRequest(HttpListenerContext ctx, string baseDir)
        {
            try
            {
                var path = ctx.Request.Url?.AbsolutePath.TrimStart('/') ?? "";
                if (string.IsNullOrEmpty(path)) path = "index.html";

                if (path.StartsWith("api/"))
                {
                    ctx.Response.ContentType = "application/json";
                    var json = "{\"status\":\"ok\",\"island\":\"NotchIsland\"}";
                    var bytes = Encoding.UTF8.GetBytes(json);
                    ctx.Response.OutputStream.Write(bytes, 0, bytes.Length);
                    ctx.Response.Close();
                    return;
                }

                var fullPath = Path.Combine(baseDir, path);
                if (File.Exists(fullPath))
                {
                    var ext = Path.GetExtension(fullPath).ToLowerInvariant();
                    ctx.Response.ContentType = ext switch
                    {
                        ".html" => "text/html; charset=utf-8",
                        ".css" => "text/css; charset=utf-8",
                        ".js" => "application/javascript; charset=utf-8",
                        ".png" => "image/png",
                        ".woff2" => "font/woff2",
                        _ => "application/octet-stream"
                    };

                    var data = File.ReadAllBytes(fullPath);
                    ctx.Response.OutputStream.Write(data, 0, data.Length);
                }
                else
                {
                    ctx.Response.StatusCode = 404;
                }
                ctx.Response.Close();
            }
            catch { }
        }

        public void Stop()
        {
            _cts?.Cancel();
            try { _listener?.Stop(); } catch { }
            IsRunning = false;
        }
    }
}
