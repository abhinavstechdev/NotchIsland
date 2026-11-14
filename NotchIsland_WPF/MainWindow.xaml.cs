using System;
using System.Diagnostics;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media.Animation;
using System.Windows.Threading;
using NotchIsland.Native;
using NotchIsland.Services;

namespace NotchIsland
{
    public partial class MainWindow : Window
    {
        private readonly MediaService _mediaService = new();
        private readonly HardwareService _hardwareService = new();
        private readonly DeckServer _deckServer = new();
        private readonly ClipboardService _clipboardService = new();
        private readonly DispatcherTimer _timer = new();

        private bool _isExpanded = false;
        private int _petAnimState = 0;
        private int _pomodoroSeconds = 25 * 60;
        private bool _pomodoroRunning = false;

        public MainWindow()
        {
            InitializeComponent();

            Closing += (s, e) =>
            {
                e.Cancel = true;
            };

            _timer.Interval = TimeSpan.FromMilliseconds(500);
            _timer.Tick += OnTimerTick;
        }

        private void Window_SourceInitialized(object sender, EventArgs e)
        {
            var hwnd = new WindowInteropHelper(this).Handle;
            Win32.MakeWindowGlass(hwnd);
            Win32.ApplyNoActivateAndTopmost(hwnd);
        }

        private async void Window_Loaded(object sender, RoutedEventArgs e)
        {
            try
            {
                // Position top center
                Left = (SystemParameters.PrimaryScreenWidth - Width) / 2;
                Top = 0;

                // Start Services
                _deckServer.Start();
                _timer.Start();

                _mediaService.TrackChanged += OnTrackChanged;
                _mediaService.PositionChanged += OnPositionChanged;
                await _mediaService.InitializeAsync();

                ClipboardList.ItemsSource = _clipboardService.Items;
            }
            catch (Exception ex)
            {
                System.IO.File.WriteAllText(@"c:\project\rea\crash.log", $"Window_Loaded error: {ex}");
            }
        }

        private void OnTimerTick(object? sender, EventArgs e)
        {
            try
            {
                _hardwareService.Update();
                _clipboardService.CheckClipboard();
                _mediaService.Tick(0.5);

                // Update Clock
                PillTime.Text = DateTime.Now.ToString("HH:mm");

                // Update Telemetry
                TelemetryText.Text = $"⚡ {_hardwareService.Stats.Fps} FPS · CPU {_hardwareService.Stats.CpuPercent}% · RAM {_hardwareService.Stats.RamUsedGb:F1}G";

                // Visualizer bars animation
                if (_mediaService.CurrentTrack.IsPlaying)
                {
                    var r = new Random();
                    Bar1.Height = r.Next(6, 14);
                    Bar2.Height = r.Next(10, 18);
                    Bar3.Height = r.Next(5, 12);
                }

                // Pomodoro countdown
                if (_pomodoroRunning && _pomodoroSeconds > 0)
                {
                    _pomodoroSeconds--;
                    var m = _pomodoroSeconds / 60;
                    var s = _pomodoroSeconds % 60;
                    TimerText.Text = $"{m:D2}:{s:D2}";
                }

                // Pet idle breathing
                _petAnimState = (_petAnimState + 1) % 10;
                if (_petAnimState == 0)
                    PetText.Text = "(=^･ω･^=)";
                else if (_petAnimState == 5)
                    PetText.Text = "(=^-ω-^=)";
            }
            catch (Exception ex)
            {
                System.IO.File.WriteAllText(@"c:\project\rea\crash.log", $"OnTimerTick error: {ex}");
            }
        }

        private void OnTrackChanged()
        {
            Dispatcher.Invoke(() =>
            {
                PillTitle.Text = $"{_mediaService.CurrentTrack.Title} · {_mediaService.CurrentTrack.Artist}";
                TrackTitle.Text = _mediaService.CurrentTrack.Title;
                TrackArtist.Text = _mediaService.CurrentTrack.Artist;
                BtnPlayPause.Content = _mediaService.CurrentTrack.IsPlaying ? "⏸" : "▶";
            });
        }

        private void OnPositionChanged()
        {
            Dispatcher.Invoke(() =>
            {
                var cur = _mediaService.CurrentTrack.Position;
                var dur = _mediaService.CurrentTrack.Duration;
                TimeElapsed.Text = $"{cur.Minutes}:{cur.Seconds:D2}";
                var rem = dur - cur;
                TimeRemaining.Text = $"-{rem.Minutes}:{rem.Seconds:D2}";

                if (dur.TotalSeconds > 0)
                {
                    MusicSeek.Value = (cur.TotalSeconds / dur.TotalSeconds) * 100.0;
                }
            });
        }

        // --- EXPAND / COLLAPSE ANIMATIONS ---
        private void IslandBody_MouseEnter(object sender, MouseEventArgs e)
        {
            ExpandIsland();
        }

        private void IslandBody_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            if (!_isExpanded) ExpandIsland();
        }

        private void ExpandIsland()
        {
            if (_isExpanded) return;
            _isExpanded = true;

            AnimateIsland(540, 260, 24);
            CompactView.Visibility = Visibility.Collapsed;
            ExpandedView.Visibility = Visibility.Visible;
        }

        private void CollapseIsland()
        {
            if (!_isExpanded) return;
            _isExpanded = false;

            AnimateIsland(230, 38, 19);
            ExpandedView.Visibility = Visibility.Collapsed;
            CompactView.Visibility = Visibility.Visible;
        }

        private void AnimateIsland(double targetW, double targetH, double targetR)
        {
            var animW = new DoubleAnimation(targetW, TimeSpan.FromMilliseconds(220)) { EasingFunction = new QuadraticEase { EasingMode = EasingMode.EaseOut } };
            var animH = new DoubleAnimation(targetH, TimeSpan.FromMilliseconds(220)) { EasingFunction = new QuadraticEase { EasingMode = EasingMode.EaseOut } };

            IslandBody.BeginAnimation(WidthProperty, animW);
            IslandBody.BeginAnimation(HeightProperty, animH);
            IslandBody.CornerRadius = new CornerRadius(targetR);
        }

        private void IslandBody_MouseLeave(object sender, MouseEventArgs e)
        {
            // Optional auto-collapse if mouse leaves window
        }

        private void CollapseBtn_Click(object sender, RoutedEventArgs e)
        {
            CollapseIsland();
        }

        // --- 10 TABS SWITCHER ---
        private void Tab_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Tag is string tag)
            {
                ContentMusic.Visibility = tag == "Music" ? Visibility.Visible : Visibility.Collapsed;
                ContentWebsites.Visibility = tag == "Websites" ? Visibility.Visible : Visibility.Collapsed;
                ContentNotifications.Visibility = tag == "Notifications" ? Visibility.Visible : Visibility.Collapsed;
                ContentShelf.Visibility = tag == "Shelf" ? Visibility.Visible : Visibility.Collapsed;
                ContentClipboard.Visibility = tag == "Clipboard" ? Visibility.Visible : Visibility.Collapsed;
                ContentWidgets.Visibility = tag == "Widgets" ? Visibility.Visible : Visibility.Collapsed;
                ContentControls.Visibility = tag == "Controls" ? Visibility.Visible : Visibility.Collapsed;
                ContentTools.Visibility = tag == "Tools" ? Visibility.Visible : Visibility.Collapsed;
                ContentDeck.Visibility = tag == "Deck" ? Visibility.Visible : Visibility.Collapsed;
                ContentCalls.Visibility = tag == "Calls" ? Visibility.Visible : Visibility.Collapsed;
            }
        }

        // --- MEDIA ACTIONS ---
        private async void MusicPlayPause_Click(object sender, RoutedEventArgs e) => await _mediaService.TogglePlayPauseAsync();
        private async void MusicNext_Click(object sender, RoutedEventArgs e) => await _mediaService.SkipNextAsync();
        private async void MusicPrev_Click(object sender, RoutedEventArgs e) => await _mediaService.SkipPreviousAsync();
        private async void MusicSeek_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (MusicSeek.IsMouseOver)
            {
                await _mediaService.SeekAsync(e.NewValue / 100.0);
            }
        }

        // --- PET INTERACTION ---
        private void PetBadge_Click(object sender, MouseButtonEventArgs e)
        {
            PetText.Text = "(=^･ω･^=) ♡";
        }

        // --- SITES & TOOLS ---
        private void LaunchSite_Click(object sender, MouseButtonEventArgs e)
        {
            if (sender is FrameworkElement el && el.Tag is string url)
            {
                Process.Start(new ProcessStartInfo(url) { UseShellExecute = true });
            }
        }

        private void AddSite_Click(object sender, MouseButtonEventArgs e)
        {
            MessageBox.Show("Add a site: Type any URL in settings to pin it here.", "NotchIsland Websites");
        }

        private void SendToPhone_Click(object sender, RoutedEventArgs e)
        {
            Process.Start(new ProcessStartInfo("http://localhost:8088") { UseShellExecute = true });
        }

        private void TimerPlay_Click(object sender, RoutedEventArgs e)
        {
            _pomodoroRunning = !_pomodoroRunning;
        }

        private void ToolScreenshot_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                Process.Start(new ProcessStartInfo("ms-screenclip:") { UseShellExecute = true });
            }
            catch { }
        }

        private void ToolSnip_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                Process.Start(new ProcessStartInfo("snippingtool.exe") { UseShellExecute = true });
            }
            catch { }
        }

        private void MicToggle_Click(object sender, RoutedEventArgs e)
        {
            MessageBox.Show("Microphone toggled (Mute/Unmute hotkey: Ctrl+Alt+M)", "NotchIsland Calls");
        }

        private void ThemeBtn_Click(object sender, RoutedEventArgs e)
        {
            MessageBox.Show("Theme switcher: OLED Pitch Black / Glass Acrylic", "NotchIsland Theme");
        }

        private void SettingsBtn_Click(object sender, RoutedEventArgs e)
        {
            MessageBox.Show("Settings: 14 categories configured (General, Appearance, Island, Taskbar, Tools, Keys, About).", "NotchIsland Settings");
        }
    }
}
