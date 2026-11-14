using System;
using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Animation;
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

        private DispatcherTimer? _fastTimer;
        private DispatcherTimer? _slowTimer;
        private Stopwatch _stopwatch = new();

        private bool _isExpanded = false;
        private bool _isMicMuted = false;
        private bool _isDraggingSeek = false;
        private int _petAnimFrame = 0;
        private double _visualizerPhase = 0;
        private Grid? _currentActivePanel;

        public MainWindow()
        {
            InitializeComponent();
            Closing += (s, e) => e.Cancel = true;
        }

        private void Window_SourceInitialized(object sender, EventArgs e)
        {
            var hwnd = new WindowInteropHelper(this).Handle;
            Win32.MakeTransparent(hwnd);
            Win32.SetWindowLongPtr(hwnd, Win32.GWL_EXSTYLE,
                (nint)(Win32.GetWindowLongPtr(hwnd, Win32.GWL_EXSTYLE) | Win32.WS_EX_TOOLWINDOW | Win32.WS_EX_TOPMOST));

            UpdatePosition();
        }

        private void UpdatePosition()
        {
            double screenWidth = SystemParameters.PrimaryScreenWidth;
            Left = (screenWidth - Width) / 2.0;
            Top = SettingsService.Current.TopGap;
        }

        private async void Window_Loaded(object sender, RoutedEventArgs e)
        {
            _currentActivePanel = PanelMusic;

            ShelfItemsControl.ItemsSource = FileShelfService.Items;
            ClipboardItemsControl.ItemsSource = _clipboard.Items;
            UpdateShelfStatus();

            await _media.InitializeAsync();
            _media.TrackChanged += OnTrackChanged;
            _media.PositionChanged += OnPositionChanged;

            _deckServer.Start();
            GenerateDeckQrCode();

            // Fast Timer (60 FPS / 16ms animation loop)
            _fastTimer = new DispatcherTimer(DispatcherPriority.Render) { Interval = TimeSpan.FromMilliseconds(16) };
            _fastTimer.Tick += FastTimer_Tick;
            _fastTimer.Start();

            // Slow Timer (1000ms update loop)
            _slowTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
            _slowTimer.Tick += SlowTimer_Tick;
            _slowTimer.Start();

            SlowTimer_Tick(null, EventArgs.Empty);
            OnTrackChanged();
        }

        #region Animation & Updates
        private void FastTimer_Tick(object? sender, EventArgs e)
        {
            _media.Tick(0.016);

            // Stopwatch
            if (_stopwatch.IsRunning)
            {
                var ts = _stopwatch.Elapsed;
                StopwatchDisplay.Text = $"{ts.Minutes:D2}:{ts.Seconds:D2}.{ts.Milliseconds / 100}";
            }

            // Smooth Harmonic Equalizer Visualizer
            _visualizerPhase += 0.15;
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

            // Dynamic Pet
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
                }
                else
                {
                    PillTitle.Text = $"NotchIsland • {now:HH:mm}";
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

        #region Fluid Morphing Animation
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

            var ease = new CubicEase { EasingMode = EasingMode.EaseOut };
            var duration = TimeSpan.FromMilliseconds(300);

            // Morph Width & Height smoothly
            var widthAnim = new DoubleAnimation(IslandContainer.ActualWidth, 560, duration) { EasingFunction = ease };
            IslandContainer.BeginAnimation(WidthProperty, widthAnim);

            var heightAnim = new DoubleAnimation(IslandContainer.ActualHeight, 270, duration) { EasingFunction = ease };
            IslandContainer.BeginAnimation(HeightProperty, heightAnim);

            // Fade Out Pill
            var fadePill = new DoubleAnimation(PillContent.Opacity, 0, TimeSpan.FromMilliseconds(120)) { EasingFunction = ease };
            PillContent.BeginAnimation(OpacityProperty, fadePill);

            // Fade In Card Content
            CardContent.IsHitTestVisible = true;
            var fadeCard = new DoubleAnimation(0, 1, TimeSpan.FromMilliseconds(200))
            {
                BeginTime = TimeSpan.FromMilliseconds(90),
                EasingFunction = ease
            };
            CardContent.BeginAnimation(OpacityProperty, fadeCard);
        }

        public void CollapseIsland()
        {
            if (!_isExpanded) return;
            _isExpanded = false;

            var ease = new CubicEase { EasingMode = EasingMode.EaseInOut };
            var duration = TimeSpan.FromMilliseconds(260);

            // Fade Out Card Content
            CardContent.IsHitTestVisible = false;
            var fadeCard = new DoubleAnimation(CardContent.Opacity, 0, TimeSpan.FromMilliseconds(90)) { EasingFunction = ease };
            CardContent.BeginAnimation(OpacityProperty, fadeCard);

            // Morph Width & Height back to Pill
            var widthAnim = new DoubleAnimation(IslandContainer.ActualWidth, 320, duration) { EasingFunction = ease };
            IslandContainer.BeginAnimation(WidthProperty, widthAnim);

            var heightAnim = new DoubleAnimation(IslandContainer.ActualHeight, 40, duration) { EasingFunction = ease };
            IslandContainer.BeginAnimation(HeightProperty, heightAnim);

            // Fade In Pill Content
            var fadePill = new DoubleAnimation(0, 1, TimeSpan.FromMilliseconds(180))
            {
                BeginTime = TimeSpan.FromMilliseconds(80),
                EasingFunction = ease
            };
            PillContent.BeginAnimation(OpacityProperty, fadePill);
        }
        #endregion

        #region Navigation Tabs with Slide & Fade Transitions
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
                "Music" => PanelMusic,
                "Shelf" => PanelShelf,
                "Clipboard" => PanelClipboard,
                "Widgets" => PanelWidgets,
                "Controls" => PanelControls,
                "Web" => PanelWeb,
                "Notifications" => PanelNotifications,
                "Tools" => PanelTools,
                "Deck" => PanelDeck,
                "Calls" => PanelCalls,
                _ => PanelMusic
            };

            if (_currentActivePanel == target) return;

            if (_currentActivePanel != null)
            {
                _currentActivePanel.Visibility = Visibility.Collapsed;
            }

            _currentActivePanel = target;
            target.Visibility = Visibility.Visible;

            if (name == "Shelf") UpdateShelfStatus();

            // Fluid slide-up and fade-in transition
            var ease = new CubicEase { EasingMode = EasingMode.EaseOut };
            var fadeIn = new DoubleAnimation(0, 1, TimeSpan.FromMilliseconds(180)) { EasingFunction = ease };
            target.BeginAnimation(OpacityProperty, fadeIn);

            if (target.RenderTransform is not TranslateTransform tt)
            {
                tt = new TranslateTransform();
                target.RenderTransform = tt;
            }
            var slideUp = new DoubleAnimation(8, 0, TimeSpan.FromMilliseconds(180)) { EasingFunction = ease };
            tt.BeginAnimation(TranslateTransform.YProperty, slideUp);
        }

        private void SettingsBtn_Click(object sender, RoutedEventArgs e)
        {
            _settingsWindow ??= new SettingsWindow();
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
            MessageBox.Show($"Copied color to clipboard: {hex}", "NotchIsland Color Picker", MessageBoxButton.OK, MessageBoxImage.Information);
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
