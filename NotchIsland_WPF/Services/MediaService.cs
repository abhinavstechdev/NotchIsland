using System;
using System.IO;
using System.Threading.Tasks;
using System.Windows.Media.Imaging;
using Windows.Media.Control;
using Windows.Storage.Streams;

namespace NotchIsland.Services
{
    public class MediaTrack
    {
        public string Title { get; set; } = "Starboy";
        public string Artist { get; set; } = "The Weeknd, Daft Punk";
        public string Album { get; set; } = "Starboy";
        public TimeSpan Duration { get; set; } = TimeSpan.FromSeconds(230);
        public TimeSpan Position { get; set; } = TimeSpan.FromSeconds(66);
        public bool IsPlaying { get; set; } = true;
        public BitmapImage? Thumbnail { get; set; }
    }

    public class MediaService
    {
        private GlobalSystemMediaTransportControlsSessionManager? _sessionManager;
        public MediaTrack CurrentTrack { get; private set; } = new MediaTrack();

        public event Action? TrackChanged;
        public event Action? PositionChanged;

        public async Task InitializeAsync()
        {
            try
            {
                _sessionManager = await GlobalSystemMediaTransportControlsSessionManager.RequestAsync();
                if (_sessionManager != null)
                {
                    _sessionManager.CurrentSessionChanged += OnCurrentSessionChanged;
                    await UpdateFromCurrentSessionAsync();
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"GSMTC Init error: {ex.Message}");
            }
        }

        private async void OnCurrentSessionChanged(GlobalSystemMediaTransportControlsSessionManager sender, CurrentSessionChangedEventArgs args)
        {
            await UpdateFromCurrentSessionAsync();
        }

        public async Task UpdateFromCurrentSessionAsync()
        {
            if (_sessionManager == null) return;

            try
            {
                var session = _sessionManager.GetCurrentSession();
                if (session != null)
                {
                    var props = await session.TryGetMediaPropertiesAsync();
                    var timeline = session.GetTimelineProperties();
                    var playback = session.GetPlaybackInfo();

                    if (props != null)
                    {
                        CurrentTrack.Title = string.IsNullOrWhiteSpace(props.Title) ? "Unknown Title" : props.Title;
                        CurrentTrack.Artist = string.IsNullOrWhiteSpace(props.Artist) ? "Unknown Artist" : props.Artist;
                        CurrentTrack.Album = props.AlbumTitle ?? "";

                        if (props.Thumbnail != null)
                        {
                            try
                            {
                                var stream = await props.Thumbnail.OpenReadAsync();
                                using var memStream = new MemoryStream();
                                await stream.AsStream().CopyToAsync(memStream);
                                memStream.Position = 0;

                                var bitmap = new BitmapImage();
                                bitmap.BeginInit();
                                bitmap.CacheOption = BitmapCacheOption.OnLoad;
                                bitmap.StreamSource = memStream;
                                bitmap.EndInit();
                                bitmap.Freeze();
                                CurrentTrack.Thumbnail = bitmap;
                            }
                            catch { }
                        }
                    }

                    if (timeline != null)
                    {
                        CurrentTrack.Duration = timeline.EndTime;
                        CurrentTrack.Position = timeline.Position;
                    }

                    if (playback != null)
                    {
                        CurrentTrack.IsPlaying = playback.PlaybackStatus == GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing;
                    }

                    TrackChanged?.Invoke();
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Error updating session: {ex.Message}");
            }
        }

        public async Task TogglePlayPauseAsync()
        {
            if (_sessionManager?.GetCurrentSession() is { } session)
            {
                await session.TryTogglePlayPauseAsync();
                await UpdateFromCurrentSessionAsync();
            }
            else
            {
                CurrentTrack.IsPlaying = !CurrentTrack.IsPlaying;
                TrackChanged?.Invoke();
            }
        }

        public async Task SkipNextAsync()
        {
            if (_sessionManager?.GetCurrentSession() is { } session)
            {
                await session.TrySkipNextAsync();
                await UpdateFromCurrentSessionAsync();
            }
            else
            {
                CurrentTrack.Position = TimeSpan.Zero;
                TrackChanged?.Invoke();
            }
        }

        public async Task SkipPreviousAsync()
        {
            if (_sessionManager?.GetCurrentSession() is { } session)
            {
                await session.TrySkipPreviousAsync();
                await UpdateFromCurrentSessionAsync();
            }
            else
            {
                CurrentTrack.Position = TimeSpan.Zero;
                TrackChanged?.Invoke();
            }
        }

        public async Task SeekAsync(double progressPercent)
        {
            var targetTicks = (long)(CurrentTrack.Duration.Ticks * Math.Clamp(progressPercent, 0.0, 1.0));
            if (_sessionManager?.GetCurrentSession() is { } session)
            {
                await session.TryChangePlaybackPositionAsync(targetTicks);
            }
            CurrentTrack.Position = TimeSpan.FromTicks(targetTicks);
            PositionChanged?.Invoke();
        }

        public void Tick(double dtSeconds)
        {
            if (CurrentTrack.IsPlaying && CurrentTrack.Duration.TotalSeconds > 0)
            {
                var newPos = CurrentTrack.Position.TotalSeconds + dtSeconds;
                if (newPos > CurrentTrack.Duration.TotalSeconds) newPos = 0;
                CurrentTrack.Position = TimeSpan.FromSeconds(newPos);
                PositionChanged?.Invoke();
            }
        }
    }
}
