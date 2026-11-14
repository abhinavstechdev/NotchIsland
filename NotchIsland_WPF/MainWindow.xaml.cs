using System;
using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using NotchIsland.Native;
using NotchIsland.Services;
using QRCoder;

namespace NotchIsland
{
    public partial class MainWindow : Window
    {
        private readonly MediaService _media = new();
        private readonly HardwareService _hardware = new();
        private readonly ClipboardService _clipboard = new();
        private readonly DeckServer _deckServer = new();
        private SettingsWindow? _settingsWindow;

        // 2nd-Order Analytical Spring Physics Engines
        private readonly AnalyticalSpring _widthSpring = new(212.0, SpringParams.CardMorph);
        private readonly AnalyticalSpring _heightSpring = new(38.0, SpringParams.CardMorph);
        private readonly AnalyticalSpring _radiusSpring = new(19.0, SpringParams.Snappy);
        private readonly AnalyticalSpring _pillAlphaSpring = new(1.0, SpringParams.Snappy);
        private readonly AnalyticalSpring _cardAlphaSpring = new(0.0, SpringParams.Snappy);
        private readonly AnalyticalSpring _tabSlideSpring = new(0.0, SpringParams.TabSlide);
        private readonly AnalyticalSpring _tabFadeSpring = new(1.0, SpringParams.TabFade);

        private DispatcherTimer? _renderLoop;
        private DispatcherTimer? _slowTimer;
        private Stopwatch _stopwatch = new();

        private bool _isExpanded = false;
        private bool _isMicMuted = false;
        private bool _isDraggingSeek = false;
        private int _petAnimFrame = 0;
        private double _visualizerPhase = 0;

        private Grid? _currentActivePanel;
        private string _currentActivePanelName = "Music";

        public MainWindow()
        {
            App.Log("MainWindow ctor enter");
            InitializeComponent();
            Closing += (s, e) => { App.Log("MainWindow Closing event triggered - cancelling"); e.Cancel = true; };
            App.Log("MainWindow ctor exit");
        }

        private void Window_SourceInitialized(object sender, EventArgs e)
        {
            App.Log("Window_SourceInitialized enter");
            try
            {
                var hwnd = new WindowInteropHelper(this).Handle;
                Win32.MakeTransparent(hwnd);
                Win32.SetWindowLongPtr(hwnd, Win32.GWL_EXSTYLE,
                    (nint)(Win32.GetWindowLongPtr(hwnd, Win32.GWL_EXSTYLE) | Win32.WS_EX_TOOLWINDOW | Win32.WS_EX_TOPMOST));

                UpdatePosition();
                App.Log("Window_SourceInitialized success");
            }
            catch (Exception ex)
            {
                App.Log($"Window_SourceInitialized exception: {ex}");
            }
        }

        private void UpdatePosition()
        {
            double screenWidth = SystemParameters.PrimaryScreenWidth;
            Left = (screenWidth - Width) / 2.0;
            Top = SettingsService.Current.TopGap;
        }

        private async void Window_Loaded(object sender, RoutedEventArgs e)
        {
            App.Log("Window_Loaded enter");
            try
            {
                _currentActivePanel = PanelMusic;

                ShelfItemsControl.ItemsSource = FileShelfService.Items;
                ClipboardItemsControl.ItemsSource = _clipboard.Items;
                UpdateShelfStatus();

                App.Log("Initializing MediaService...");
                await _media.InitializeAsync();
                _media.TrackChanged += OnTrackChanged;
                _media.PositionChanged += OnPositionChanged;
                App.Log("MediaService initialized");

                App.Log("Starting DeckServer...");
                _deckServer.Start();
                GenerateDeckQrCode();
                App.Log("DeckServer started");

                // 60 FPS Analytical Physics & Render Loop (16ms)
                _renderLoop = new DispatcherTimer(DispatcherPriority.Render) { Interval = TimeSpan.FromMilliseconds(16) };
                _renderLoop.Tick += RenderLoop_Tick;
                _renderLoop.Start();
                App.Log("RenderLoop started");

                // 1 Hz Background Telemetry Loop
                _slowTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
                _slowTimer.Tick += SlowTimer_Tick;
                _slowTimer.Start();
                App.Log("SlowTimer started");

                SlowTimer_Tick(null, EventArgs.Empty);
                OnTrackChanged();
                App.Log("Window_Loaded completed successfully");
            }
            catch (Exception ex)
            {
                App.Log($"Window_Loaded CRITICAL ERROR: {ex}");
            }
        }

        #region Analytical Physics & 60 FPS Render Dispatcher
        private void RenderLoop_Tick(object? sender, EventArgs e)
        {
            const double dt = 0.016;

            // 1. Analytical Spring Differential Solvers
            _widthSpring.Update(dt);
            _heightSpring.Update(dt);
            _radiusSpring.Update(dt);
            _pillAlphaSpring.Update(dt);
            _cardAlphaSpring.Update(dt);
            _tabSlideSpring.Update(dt);
            _tabFadeSpring.Update(dt);

            // 2. Direct Container Geometry & Alpha Update
            IslandContainer.Width = _widthSpring.Value;
            IslandContainer.Height = _heightSpring.Value;
            IslandContainer.CornerRadius = new CornerRadius(_radiusSpring.Value);

            PillContent.Opacity = _pillAlphaSpring.Value;
            CardContent.Opacity = _cardAlphaSpring.Value;
            CardContent.IsHitTestVisible = _cardAlphaSpring.Value > 0.4;

            // 3. Tab Slide & Fade Update
            if (_currentActivePanel != null)
            {
                _currentActivePanel.Opacity = _tabFadeSpring.Value;
                if (_currentActivePanel.RenderTransform is TranslateTransform tt)
                {
                    tt.X = _tabSlideSpring.Value;
                }
                else
                {
                    _currentActivePanel.RenderTransform = new TranslateTransform(_tabSlideSpring.Value, 0);
                }
            }

            // 4. Media Tick & Stopwatch
            _media.Tick(dt);
            if (_stopwatch.IsRunning)
            {
                var ts = _stopwatch.Elapsed;
                StopwatchDisplay.Text = $"{ts.Minutes:D2}:{ts.Seconds:D2}.{ts.Milliseconds / 100}";
            }

            // 5. Harmonic Equalizer Physics (Visualizer: "pixels")
            _visualizerPhase += 0.16;
            if (_media.CurrentTrack.IsPlaying)
            {
                Bar1.Height = 5 + Math.Abs(Math.Sin(_visualizerPhase * 1.2)) * 12;
                Bar2.Height = 7 + Math.Abs(Math.Sin(_visualizerPhase * 1.8 + 1.0)) * 14;
                Bar3.Height = 4 + Math.Abs(Math.Sin(_visualizerPhase * 1.4 + 2.2)) * 11;
                Bar4.Height = 8 + Math.Abs(Math.Sin(_visualizerPhase * 2.2 + 0.6)) * 15;
            }
            else
            {
                Bar1.Height = 4;
                Bar2.Height = 4;
                Bar3.Height = 4;
                Bar4.Height = 4;
            }

            // 6. Desktop Pet Frame Switch
            _petAnimFrame = (_petAnimFrame + 1) % 60;
            if (_petAnimFrame == 0) PillPet.Text = "(=^･ω･^=)";
            else if (_petAnimFrame == 30) PillPet.Text = "(=^･ｪ･^=)";
        }

        private void SlowTimer_Tick(object? sender, EventArgs e)
        {
            var now = DateTime.Now;

            WidgetClock.Text = now.ToString("HH:mm:ss");
            WidgetDate.Text = now.ToString("dddd, MMMM dd, yyyy");

            _hardware.Update();
            WidgetCpu.Text = $"CPU: {_hardware.Stats.CpuPercent}%";
            WidgetRam.Text = $"RAM: {_hardware.Stats.RamUsedGb} / {_hardware.Stats.RamTotalGb} GB";
            WidgetBattery.Text = $"{_hardware.Stats.BatteryPercent}% {(_hardware.Stats.IsCharging ? "⚡" : "")}";

            PillBattery.Text = $"{_hardware.Stats.BatteryPercent}% {(_hardware.Stats.IsCharging ? "⚡" : "")}";
            if (!_isExpanded)
            {
                if (_media.CurrentTrack.IsPlaying && !string.IsNullOrWhiteSpace(_media.CurrentTrack.Title))
                {
                    PillTitle.Text = $"{_media.CurrentTrack.Title} • {_media.CurrentTrack.Artist}";
                    _widthSpring.SetTarget(280.0);
                }
                else
                {
                    PillTitle.Text = $"NotchIsland • {now:HH:mm}";
                    _widthSpring.SetTarget(212.0);
                }
            }

            _clipboard.CheckClipboard();

            VolumeSlider.Value = _hardware.Stats.Volume;
            VolumeLabel.Text = $"{_hardware.Stats.Volume}%";
            BrightnessSlider.Value = _hardware.Stats.Brightness;
            BrightnessLabel.Text = $"{_hardware.Stats.Brightness}%";
        }

        private void OnTrackChanged()
        {
            Dispatcher.Invoke(() =>
            {
                var t = _media.CurrentTrack;
                TrackTitle.Text = t.Title;
                TrackArtist.Text = $"{t.Artist} • {t.Album}";
                TimeTotal.Text = $"{t.Duration.Minutes:D2}:{t.Duration.Seconds:D2}";
                PlayPauseBtn.Content = t.IsPlaying ? "⏸" : "▶";

                if (t.Thumbnail != null)
                {
                    AlbumArtImage.Source = t.Thumbnail;
                }

                if (!_isExpanded && t.IsPlaying)
                {
                    PillTitle.Text = $"{t.Title} • {t.Artist}";
                    _widthSpring.SetTarget(280.0);
                }
            });
        }

        private void OnPositionChanged()
        {
            Dispatcher.Invoke(() =>
            {
                var t = _media.CurrentTrack;
                TimeElapsed.Text = $"{t.Position.Minutes:D2}:{t.Position.Seconds:D2}";

                if (!_isDraggingSeek && t.Duration.TotalSeconds > 0)
                {
                    SeekSlider.Value = (t.Position.TotalSeconds / t.Duration.TotalSeconds) * 100.0;
                }
            });
        }
        #endregion

        #region Morphing State Machine (Pill <-> Card)
        private void Island_MouseEnter(object sender, MouseEventArgs e) { }
        private void Island_MouseLeave(object sender, MouseEventArgs e) { }

        private void Island_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            if (!_isExpanded)
            {
                ExpandIsland();
            }
        }

        private void CollapseBtn_Click(object sender, RoutedEventArgs e)
        {
            CollapseIsland();
        }

        public void ExpandIsland()
        {
            if (_isExpanded) return;
            _isExpanded = true;

            _widthSpring.Params = SpringParams.CardMorph;
            _widthSpring.SetTarget(540.0);

            _heightSpring.Params = SpringParams.CardMorph;
            _heightSpring.SetTarget(270.0);

            _radiusSpring.Params = SpringParams.Snappy;
            _radiusSpring.SetTarget(24.0);

            _pillAlphaSpring.Params = SpringParams.Snappy;
            _pillAlphaSpring.SetTarget(0.0);

            _cardAlphaSpring.Params = SpringParams.Snappy;
            _cardAlphaSpring.SetTarget(1.0);
        }

        public void CollapseIsland()
        {
            if (!_isExpanded) return;
            _isExpanded = false;

            double targetW = (_media.CurrentTrack.IsPlaying && !string.IsNullOrWhiteSpace(_media.CurrentTrack.Title)) ? 280.0 : 212.0;

            _widthSpring.Params = SpringParams.Snappy;
            _widthSpring.SetTarget(targetW);

            _heightSpring.Params = SpringParams.Snappy;
            _heightSpring.SetTarget(38.0);

            _radiusSpring.Params = SpringParams.Snappy;
            _radiusSpring.SetTarget(19.0);

            _cardAlphaSpring.Params = SpringParams.Snappy;
            _cardAlphaSpring.SetTarget(0.0);

            _pillAlphaSpring.Params = SpringParams.Snappy;
            _pillAlphaSpring.SetTarget(1.0);
        }
        #endregion

        #region Tab Transitions with Directional Glide
        private void Tab_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Tag is string tabName)
            {
                ShowPanel(tabName);
            }
        }

        private void ShowPanel(string name)
        {
            Grid target = name switch
            {
                "Music" or "Media" => PanelMusic,
                "Web" => PanelWeb,
                "Notifications" => PanelNotifications,
                "Chat" => PanelChat,
                "Agents" => PanelAgents,
                "Shelf" => PanelShelf,
                "Clipboard" => PanelClipboard,
                "Widgets" => PanelWidgets,
                "Controls" => PanelControls,
                "Tools" => PanelTools,
                "Deck" => PanelDeck,
                "Tiles" => PanelTiles,
                "Calls" => PanelCalls,
                _ => PanelMusic
            };

            if (_currentActivePanel == target) return;

            int targetIndex = GetTabIndex(name);
            int currentIndex = GetTabIndex(_currentActivePanelName);
            double slideDir = targetIndex >= currentIndex ? 24.0 : -24.0;

            _currentActivePanelName = name;

            if (_currentActivePanel != null)
            {
                _currentActivePanel.Visibility = Visibility.Collapsed;
            }

            _currentActivePanel = target;
            target.Visibility = Visibility.Visible;

            if (name == "Shelf") UpdateShelfStatus();

            // Trigger Spring Slide & Fade
            _tabSlideSpring.SnapTo(slideDir);
            _tabSlideSpring.SetTarget(0.0);

            _tabFadeSpring.SnapTo(0.0);
            _tabFadeSpring.SetTarget(1.0);
        }

        private static int GetTabIndex(string name) => name switch
        {
            "Music" or "Media" => 0,
            "Web" => 1,
            "Notifications" => 2,
            "Chat" => 3,
            "Agents" => 4,
            "Shelf" => 5,
            "Clipboard" => 6,
            "Widgets" => 7,
            "Controls" => 8,
            "Tools" => 9,
            "Deck" => 10,
            "Tiles" => 11,
            "Calls" => 12,
            _ => 0
        };

        private void SettingsBtn_Click(object sender, RoutedEventArgs e)
        {
            if (_settingsWindow == null || !_settingsWindow.IsLoaded)
            {
                _settingsWindow = new SettingsWindow();
            }
            _settingsWindow.Show();
            _settingsWindow.Activate();
        }
        #endregion

        #region Media Actions
        private async void PlayPauseBtn_Click(object sender, RoutedEventArgs e)
        {
            await _media.TogglePlayPauseAsync();
        }

        private async void PrevBtn_Click(object sender, RoutedEventArgs e)
        {
            await _media.SkipPreviousAsync();
        }

        private async void NextBtn_Click(object sender, RoutedEventArgs e)
        {
            await _media.SkipNextAsync();
        }

        private async void SeekSlider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (SeekSlider.IsMouseCaptureWithin)
            {
                _isDraggingSeek = true;
                double frac = SeekSlider.Value / 100.0;
                await _media.SeekAsync(frac);
                _isDraggingSeek = false;
            }
        }
        #endregion

        #region File Shelf Actions
        private void Shelf_DragOver(object sender, DragEventArgs e)
        {
            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                e.Effects = DragDropEffects.Copy;
                e.Handled = true;
            }
        }

        private void Shelf_Drop(object sender, DragEventArgs e)
        {
            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                var files = (string[])e.Data.GetData(DataFormats.FileDrop);
                foreach (var f in files)
                {
                    FileShelfService.AddFile(f);
                }
                UpdateShelfStatus();
            }
        }

        private void ShelfItemOpen_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Tag is ShelfItem item)
            {
                FileShelfService.OpenFile(item);
            }
        }

        private void ShelfItemRemove_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Tag is ShelfItem item)
            {
                FileShelfService.RemoveItem(item);
                UpdateShelfStatus();
            }
        }

        private void ShelfClear_Click(object sender, RoutedEventArgs e)
        {
            FileShelfService.ClearAll();
            UpdateShelfStatus();
        }

        private void UpdateShelfStatus()
        {
            ShelfStatus.Text = $"Shelf holds {FileShelfService.Items.Count} files";
        }
        #endregion

        #region Clipboard Actions
        private void ClipboardItem_Click(object sender, MouseButtonEventArgs e)
        {
            if (sender is FrameworkElement fe && fe.DataContext is ClipboardItem item)
            {
                _clipboard.CopyItem(item);
            }
        }

        private void ClipboardDelete_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Tag is ClipboardItem item)
            {
                _clipboard.DeleteItem(item);
            }
        }

        private void ClipboardClear_Click(object sender, RoutedEventArgs e)
        {
            _clipboard.ClearAll();
        }
        #endregion

        #region Widgets Actions
        private void TimerToggle_Click(object sender, RoutedEventArgs e)
        {
            if (_stopwatch.IsRunning)
            {
                _stopwatch.Stop();
                TimerToggleBtn.Content = "▶";
            }
            else
            {
                _stopwatch.Start();
                TimerToggleBtn.Content = "⏸";
            }
        }

        private void TimerReset_Click(object sender, RoutedEventArgs e)
        {
            _stopwatch.Reset();
            StopwatchDisplay.Text = "00:00.0";
            TimerToggleBtn.Content = "▶";
        }
        #endregion

        #region Controls Actions
        private void VolumeSlider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (VolumeSlider.IsMouseCaptureWithin)
            {
                SystemControlService.SetMasterVolume((float)VolumeSlider.Value);
                VolumeLabel.Text = $"{(int)VolumeSlider.Value}%";
            }
        }

        private void MuteBtn_Click(object sender, RoutedEventArgs e)
        {
            SystemControlService.ToggleMute();
            bool muted = SystemControlService.GetIsMuted();
            MuteBtn.Content = muted ? "Unmute" : "Mute";
        }

        private void BrightnessSlider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (BrightnessSlider.IsMouseCaptureWithin)
            {
                SystemControlService.SetBrightness((int)BrightnessSlider.Value);
                BrightnessLabel.Text = $"{(int)BrightnessSlider.Value}%";
            }
        }

        private void WifiBtn_Click(object sender, RoutedEventArgs e)
        {
            SystemControlService.OpenNetworkSettings();
        }

        private void BluetoothBtn_Click(object sender, RoutedEventArgs e)
        {
            SystemControlService.OpenBluetoothSettings();
        }

        private void NightLightBtn_Click(object sender, RoutedEventArgs e)
        {
            SystemControlService.OpenNightLightSettings();
        }
        #endregion

        #region Web Actions
        private void WebTile_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Tag is string url)
            {
                try { Process.Start(new ProcessStartInfo(url) { UseShellExecute = true }); } catch { }
            }
        }

        private void WebLaunchCustom_Click(object sender, RoutedEventArgs e)
        {
            string url = WebUrlInput.Text.Trim();
            if (!url.StartsWith("http://") && !url.StartsWith("https://"))
            {
                url = "https://" + url;
            }
            try { Process.Start(new ProcessStartInfo(url) { UseShellExecute = true }); } catch { }
        }
        #endregion

        #region Notifications Actions
        private void NotificationsClear_Click(object sender, RoutedEventArgs e)
        {
            NotificationsStack.Children.Clear();
            NotificationsStack.Children.Add(new TextBlock
            {
                Text = "No notifications",
                Foreground = Brushes.Gray,
                FontSize = 12,
                Margin = new Thickness(0, 10, 0, 0),
                HorizontalAlignment = HorizontalAlignment.Center
            });
        }
        #endregion

        #region Tools Actions
        private void ScreenshotBtn_Click(object sender, RoutedEventArgs e)
        {
            var saved = SystemControlService.TakeScreenshot();
            if (saved != null)
            {
                UpdateShelfStatus();
            }
        }

        private void SnipBtn_Click(object sender, RoutedEventArgs e)
        {
            SystemControlService.LaunchSnip();
        }

        private void ColorPickerBtn_Click(object sender, RoutedEventArgs e)
        {
            string hex = SystemControlService.PickColorUnderCursor();
            MessageBox.Show($"Copied color to clipboard: {hex}", "NotchPixel Color Picker", MessageBoxButton.OK, MessageBoxImage.Information);
        }

        private void TaskManagerBtn_Click(object sender, RoutedEventArgs e)
        {
            try { Process.Start(new ProcessStartInfo("taskmgr.exe") { UseShellExecute = true }); } catch { }
        }
        #endregion

        #region Deck Actions
        private void GenerateDeckQrCode()
        {
            try
            {
                using var generator = new QRCodeGenerator();
                using var data = generator.CreateQrCode("http://localhost:8088", QRCodeGenerator.ECCLevel.Q);
                using var qrCode = new PngByteQRCode(data);
                byte[] qrBytes = qrCode.GetGraphic(20);

                using var stream = new MemoryStream(qrBytes);
                var bmp = new BitmapImage();
                bmp.BeginInit();
                bmp.CacheOption = BitmapCacheOption.OnLoad;
                bmp.StreamSource = stream;
                bmp.EndInit();
                bmp.Freeze();

                DeckQrCode.Source = bmp;
            }
            catch { }
        }

        private void DeckOpenBrowser_Click(object sender, RoutedEventArgs e)
        {
            try { Process.Start(new ProcessStartInfo("http://localhost:8088") { UseShellExecute = true }); } catch { }
        }
        #endregion

        #region Calls Actions
        private void MicMuteCallBtn_Click(object sender, RoutedEventArgs e)
        {
            _isMicMuted = !_isMicMuted;
            if (_isMicMuted)
            {
                MicMuteIcon.Text = "🔇";
                MicMuteStatus.Text = "Microphone MUTED (Click to Unmute)";
                MicMuteCallBtn.Background = (Brush)new BrushConverter().ConvertFrom("#40FF3B30")!;
            }
            else
            {
                MicMuteIcon.Text = "🎤";
                MicMuteStatus.Text = "Microphone LIVE (Click to Mute)";
                MicMuteCallBtn.Background = (Brush)new BrushConverter().ConvertFrom("#16FFFFFF")!;
            }
        }
        #endregion
    }
}
