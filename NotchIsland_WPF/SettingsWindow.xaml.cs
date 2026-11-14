using System;
using System.Diagnostics;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using NotchIsland.Services;

namespace NotchIsland
{
    public partial class SettingsWindow : Window
    {
        public SettingsWindow()
        {
            InitializeComponent();
            Closing += (s, e) => { e.Cancel = true; Hide(); };
            ShowCategory("General");
        }

        private void TitleBar_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            if (e.ButtonState == MouseButtonState.Pressed)
            {
                DragMove();
            }
        }

        private void CloseBtn_Click(object sender, RoutedEventArgs e)
        {
            Hide();
        }

        private void Nav_Checked(object sender, RoutedEventArgs e)
        {
            if (sender is RadioButton rb && rb.Tag is string tag)
            {
                ShowCategory(tag);
            }
        }

        private void ShowCategory(string category)
        {
            if (ContentStack == null) return;
            ContentStack.Children.Clear();
            var s = SettingsService.Current;

            switch (category)
            {
                case "General":
                    AddHeader("General", "Island layout, placement, and auto-behavior");
                    AddToggle("Slim when covered", "Island shrinks to a slim line when an active window maximizes", s.SlimWhenCovered, v => { s.SlimWhenCovered = v; Save(); });
                    AddToggle("Hide in fullscreen", "Automatically hide the island in fullscreen games or media", s.HideInFullscreen, v => { s.HideInFullscreen = v; Save(); });
                    AddToggle("Hide from screen captures", "Exclude island from desktop screenshots and OBS recordings", s.HideFromCaptures, v => { s.HideFromCaptures = v; Save(); });
                    AddSlider("Hover Delay (ms)", "Time required before island expands on cursor hover", s.HoverDelayMs, 0, 500, 10, v => { s.HoverDelayMs = (int)v; Save(); });
                    AddSlider("Top Gap (pixels)", "Vertical offset from top edge of the primary monitor", s.TopGap, 0, 30, 1, v => { s.TopGap = (int)v; s.TopGapDip = (int)v; Save(); });
                    AddSlider("Edge Margin (pixels)", "Horizontal clearance from display bezels", s.EdgeMargin, 0, 100, 4, v => { s.EdgeMargin = (int)v; Save(); });
                    break;

                case "Accessibility":
                    AddHeader("Accessibility", "Contrast, reduced motion, and readability options");
                    AddToggle("High Contrast Text", "Use high-contrast stark white text for enhanced legibility", s.HighContrastText, v => { s.HighContrastText = v; Save(); });
                    AddToggle("Reduce Motion", "Disable springs and micro-animations for instant transitions", s.ReduceMotion, v => { s.ReduceMotion = v; Save(); });
                    AddToggle("Screen Reader Announcements", "Announce active island panel changes to UI automation", s.ScreenReaderAnnounce, v => { s.ScreenReaderAnnounce = v; Save(); });
                    AddSlider("Text Scale Multiplier", "Relative font size multiplier across all island cards", s.FontScale, 0.8, 1.5, 0.05, v => { s.FontScale = Math.Round(v, 2); Save(); });
                    break;

                case "Appearance":
                    AddHeader("Appearance", "Surface styling, OLED palette, blur, and spring physics");
                    AddToggle("OLED Deep Black", "Pure zero-backlight OLED black (#000000) backdrop", s.Surface == "black", v => { s.Surface = v ? "black" : "glass"; Save(); });
                    AddSlider("Glass Blur Strength", "Backdrop acrylic / frosted glass blur radius", s.GlassBlur, 0, 100, 5, v => { s.GlassBlur = (int)v; Save(); });
                    AddSlider("Corner Roundness (%)", "Squircle roundness percentage (100% = full capsule)", s.CornerRoundness, 50, 100, 5, v => { s.CornerRoundness = (int)v; Save(); });
                    AddSlider("Island Scale (%)", "Scale ratio for compact pill and expanded cards", s.Size, 80, 140, 5, v => { s.Size = (int)v; Save(); });
                    AddToggle("Wave Progress Bar", "Fluid continuous wave animation on scrub slider", s.ProgressBar == "wave", v => { s.ProgressBar = v ? "wave" : "flat"; Save(); });
                    AddToggle("Album Art Backdrop", "Vibrant dynamic glow matching current playing track", s.MusicBackground == "album", v => { s.MusicBackground = v ? "album" : "dark"; Save(); });
                    break;

                case "Island":
                    AddHeader("Island & Shape", "Notch vs floating capsule geometry and idle desktop pets");
                    AddToggle("Physical Notch Flare", "Render reverse flare ears joining top display bezel", s.NotchEarFlare, v => { s.NotchEarFlare = v; Save(); });
                    AddToggle("Show Desktop Pet When Idle", "Display animated pixel cat in left island slot", s.IdleShows == "pet", v => { s.IdleShows = v ? "pet" : "empty"; Save(); });
                    AddInfoCard("Island Shape Mode", s.Shape + " (Center Top Edge)");
                    AddInfoCard("Active Motion Profile", s.Motion + " (2nd-Order Harmonic Oscillator)");
                    break;

                case "Taskbar":
                    AddHeader("Taskbar Integration", "Taskbar docking and multi-monitor AppBar integration");
                    AddToggle("Taskbar Island Integration", "Attach island to top or bottom taskbar automatically", s.TaskbarIslandIntegration, v => { s.TaskbarIslandIntegration = v; Save(); });
                    AddToggle("Taskbar Hover Expand", "Expand card when hovering taskbar island badge", s.TaskbarHoverExpand, v => { s.TaskbarHoverExpand = v; Save(); });
                    AddToggle("Auto-Position Over Taskbar", "Avoid overlapping pinned system taskbars", s.AutoPositionOverTaskbar, v => { s.AutoPositionOverTaskbar = v; Save(); });
                    break;

                case "Panels":
                    AddHeader("Panels Manager", "Toggle visibility of all 14 card panels and tabs");
                    AddToggle("Media Player Panel", "Track controls, wave scrubber, volume, and lyrics", s.MediaPanel, v => { s.MediaPanel = v; Save(); });
                    AddToggle("File Shelf Panel", "Drag-and-drop staging shelf with send-to-phone QR", s.ShelfEnabled, v => { s.ShelfEnabled = v; Save(); });
                    AddToggle("Clipboard History Panel", "Persistent snippet history and image pinboard", s.ClipboardEnabled, v => { s.ClipboardEnabled = v; Save(); });
                    AddToggle("System Controls Panel", "Dual sliders for volume & brightness, quick toggles", s.ControlsPanel, v => { s.ControlsPanel = v; Save(); });
                    AddToggle("Websites Quick Panel", "Pinned web app cards (YouTube, Spotify, etc.)", s.WebPanel, v => { s.WebPanel = v; Save(); });
                    AddToggle("Notifications Panel", "Mirrored Windows push notifications carousel", s.NotificationsPanel, v => { s.NotificationsPanel = v; Save(); });
                    AddToggle("App Launcher Panel", "Quick launch slots for frequently used apps", s.LauncherPanel, v => { s.LauncherPanel = v; Save(); });
                    AddToggle("PC Deck Panel", "Mobile browser companion integration", s.DeckPanel, v => { s.DeckPanel = v; Save(); });
                    break;

                case "Widgets":
                    AddHeader("Widgets (4-Bento Grid)", "Calendar, Pomodoro timer, weather, and hardware stats");
                    AddToggle("Calendar & Date Bento", "Displays current date, day, and upcoming events", s.ShowDateWidget, v => { s.ShowDateWidget = v; Save(); });
                    AddToggle("Pomodoro Focus Timer", "Customizable focus and break interval timer", s.ShowTimerWidget, v => { s.ShowTimerWidget = v; Save(); });
                    AddSlider("Focus Minutes", "Pomodoro work session duration", s.FocusMinutes, 5, 60, 5, v => { s.FocusMinutes = (int)v; Save(); });
                    AddSlider("Break Minutes", "Pomodoro short break duration", s.BreakMinutes, 1, 15, 1, v => { s.BreakMinutes = (int)v; Save(); });
                    AddToggle("Weather Bento", "Real-time forecast, temperature, and condition icon", s.ShowWeatherWidget, v => { s.ShowWeatherWidget = v; Save(); });
                    AddToggle("Hardware Telemetry Bento", "Real-time CPU, GPU, RAM, and network usage", s.ShowStatsWidget, v => { s.ShowStatsWidget = v; Save(); });
                    break;

                case "Calls":
                    AddHeader("Calls & Hardware Audio", "Meeting microphone toggle, output selector, and live input test");
                    AddToggle("Auto-Detect Active Calls", "Show LiveActivity pill when Teams, Zoom, or Meet starts", s.AutoDetectCalls, v => { s.AutoDetectCalls = v; Save(); });
                    AddToggle("Enable Global Mic Mute Shortcut", "Use hotkey to mute system microphone everywhere", s.MicMuteShortcutEnabled, v => { s.MicMuteShortcutEnabled = v; Save(); });
                    AddInfoCard("Microphone Test", "Click Hero Mic Button in island card to test mute and waveform");
                    AddInfoCard("Active Microphone Input", s.PreferredAudioInput);
                    AddInfoCard("Active Audio Output", s.PreferredAudioOutput);
                    break;

                case "Streaming":
                    AddHeader("Streaming & OBS", "Broadcast status, live indicator, and streaming hotkeys");
                    AddToggle("OBS Studio Integration", "Sync live on-air status with OBS WebSocket", s.ObsIntegration, v => { s.ObsIntegration = v; Save(); });
                    AddToggle("Show LIVE Indicator Pill", "Pulsing red recording pill on island during stream", s.LiveIndicator, v => { s.LiveIndicator = v; Save(); });
                    AddToggle("Sync Stream Mute Status", "Show mute state during active streams", s.StreamingMuteStatus, v => { s.StreamingMuteStatus = v; Save(); });
                    break;

                case "Gaming":
                    AddHeader("Gaming Mode", "Low-profile telemetry HUD and auto-suppression");
                    AddToggle("Auto Game Detection", "Detect DirectX/Vulkan games and switch to slim HUD", s.GameDetection, v => { s.GameDetection = v; Save(); });
                    AddToggle("Slim Gaming HUD (380x28px)", "Ultra-compact non-intrusive status bar at top edge", s.SlimGamingHud, v => { s.SlimGamingHud = v; Save(); });
                    AddToggle("Display FPS Counter", "Show live game frame rate in gaming HUD", s.ShowFpsCounter, v => { s.ShowFpsCounter = v; Save(); });
                    break;

                case "Agents":
                    AddHeader("AI Agents LiveActivity", "Active task telemetry for Claude, ChatGPT, and local models");
                    AddToggle("Claude Code / ADK Integration", "Show live thinking status, tool calls, and run state", s.ClaudeAgentEnabled, v => { s.ClaudeAgentEnabled = v; Save(); });
                    AddToggle("ChatGPT LiveActivity", "Display response stream status and voice activity", s.ChatGptAgentEnabled, v => { s.ChatGptAgentEnabled = v; Save(); });
                    AddToggle("Show Agent LiveActivity Pill", "Expand capsule with animated spark icon while running", s.ShowAgentLiveActivity, v => { s.ShowAgentLiveActivity = v; Save(); });
                    break;

                case "Tools":
                    AddHeader("Tools & Windows HUD Replacements", "Hardware flyouts, screenshots, and screen recording");
                    AddToggle("Replace Windows Volume Flyout", "Show sleek island HUD instead of bulky Windows box", s.ReplaceVolumeFlyout, v => { s.ReplaceVolumeFlyout = v; Save(); });
                    AddToggle("Show Volume HUD", "Display volume level changes in island capsule", s.ShowVolumeHud, v => { s.ShowVolumeHud = v; Save(); });
                    AddToggle("Show Brightness HUD", "Display brightness slider pill on display changes", s.ShowBrightnessHud, v => { s.ShowBrightnessHud = v; Save(); });
                    AddToggle("Show Battery & Power HUD", "Show charging alerts and low battery warnings", s.ShowBatteryHud, v => { s.ShowBatteryHud = v; Save(); });
                    AddToggle("Show Lock Keys HUD", "Show Caps Lock / Num Lock alerts", s.ShowLockKeysHud, v => { s.ShowLockKeysHud = v; Save(); });
                    AddToggle("Mirror Windows Notifications", "Mirrors desktop toast alerts inside island", s.MirrorNotifications, v => { s.MirrorNotifications = v; Save(); });
                    AddSlider("Notification Timeout (seconds)", "How long toasts remain visible", s.NotificationSeconds, 2, 15, 1, v => { s.NotificationSeconds = (int)v; Save(); });
                    break;

                case "Keys":
                    AddHeader("Global Hotkeys", "Keyboard shortcuts for instant hands-free control");
                    AddInfoCard("Toggle Island Card", s.HotkeyToggleCard);
                    AddInfoCard("Play / Pause Media", s.HotkeyPlayPause);
                    AddInfoCard("Focus Timer / Stopwatch", s.HotkeyTimer);
                    AddInfoCard("Hide / Unhide Island", s.HotkeyHide);
                    AddInfoCard("Toggle Microphone Mute", s.HotkeyMicMute);
                    AddInfoCard("Instant Screenshot", s.HotkeyScreenshot);
                    AddInfoCard("Area Snip Tool", "ms-screenclip: (Win + Shift + S)");
                    AddInfoCard("Screen Video Record", s.HotkeyScreenRecord);
                    break;

                case "About":
                    AddHeader("About NotchIsland", "100% Native Windows Dynamic Island Utility");
                    AddInfoCard("Version", "0.5.108 (Native x64 Release)");
                    AddInfoCard("Runtime", ".NET 9 + DirectComposition / SkiaSharp Continuous Squircle");
                    AddInfoCard("Physics Engine", "2nd-Order Analytical Damped Harmonic Oscillator (RK / Rs)");
                    AddInfoCard("License", "100% Free & Open Source Software");
                    AddButton("Open PC Deck Companion in Browser", "Launch local HTTP deck at http://localhost:8088", () =>
                    {
                        try { Process.Start(new ProcessStartInfo("http://localhost:8088") { UseShellExecute = true }); } catch { }
                    });
                    AddButton("Visit GitHub Repository", "Open project repository in default browser", () =>
                    {
                        try { Process.Start(new ProcessStartInfo("https://github.com/abhinavstechdev/NotchIsland") { UseShellExecute = true }); } catch { }
                    });
                    break;
            }
        }

        private void Save()
        {
            SettingsService.Save();
        }

        private void AddHeader(string title, string subtitle)
        {
            var sp = new StackPanel { Margin = new Thickness(0, 0, 0, 16) };
            sp.Children.Add(new TextBlock
            {
                Text = title,
                Foreground = Brushes.White,
                FontSize = 18,
                FontWeight = FontWeights.Bold
            });
            sp.Children.Add(new TextBlock
            {
                Text = subtitle,
                Foreground = (Brush)new BrushConverter().ConvertFrom("#88FFFFFF")!,
                FontSize = 13,
                Margin = new Thickness(0, 4, 0, 0)
            });
            ContentStack.Children.Add(sp);
        }

        private void AddToggle(string title, string description, bool initialValue, Action<bool> onToggled)
        {
            var card = new Border
            {
                Background = (Brush)new BrushConverter().ConvertFrom("#121215")!,
                BorderBrush = (Brush)new BrushConverter().ConvertFrom("#1FFFFFFF")!,
                BorderThickness = new Thickness(1),
                CornerRadius = new CornerRadius(10),
                Padding = new Thickness(16, 12, 16, 12),
                Margin = new Thickness(0, 0, 0, 10)
            };

            var grid = new Grid();
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

            var sp = new StackPanel { VerticalAlignment = VerticalAlignment.Center };
            sp.Children.Add(new TextBlock { Text = title, Foreground = Brushes.White, FontSize = 13.5, FontWeight = FontWeights.SemiBold });
            sp.Children.Add(new TextBlock { Text = description, Foreground = (Brush)new BrushConverter().ConvertFrom("#88FFFFFF")!, FontSize = 12, Margin = new Thickness(0, 2, 0, 0) });
            Grid.SetColumn(sp, 0);
            grid.Children.Add(sp);

            var chk = new CheckBox
            {
                IsChecked = initialValue,
                VerticalAlignment = VerticalAlignment.Center,
                Cursor = Cursors.Hand
            };
            chk.Checked += (s, e) => onToggled(true);
            chk.Unchecked += (s, e) => onToggled(false);
            Grid.SetColumn(chk, 1);
            grid.Children.Add(chk);

            card.Child = grid;
            ContentStack.Children.Add(card);
        }

        private void AddSlider(string title, string description, double value, double min, double max, double tick, Action<double> onChanged)
        {
            var card = new Border
            {
                Background = (Brush)new BrushConverter().ConvertFrom("#121215")!,
                BorderBrush = (Brush)new BrushConverter().ConvertFrom("#1FFFFFFF")!,
                BorderThickness = new Thickness(1),
                CornerRadius = new CornerRadius(10),
                Padding = new Thickness(16, 12, 16, 12),
                Margin = new Thickness(0, 0, 0, 10)
            };

            var sp = new StackPanel();
            var headerGrid = new Grid();
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

            var textSp = new StackPanel();
            textSp.Children.Add(new TextBlock { Text = title, Foreground = Brushes.White, FontSize = 13.5, FontWeight = FontWeights.SemiBold });
            textSp.Children.Add(new TextBlock { Text = description, Foreground = (Brush)new BrushConverter().ConvertFrom("#88FFFFFF")!, FontSize = 12, Margin = new Thickness(0, 2, 0, 0) });
            Grid.SetColumn(textSp, 0);
            headerGrid.Children.Add(textSp);

            var valText = new TextBlock
            {
                Text = value.ToString("0.##"),
                Foreground = (Brush)new BrushConverter().ConvertFrom("#64D2FF")!,
                FontSize = 13,
                FontWeight = FontWeights.SemiBold,
                VerticalAlignment = VerticalAlignment.Center
            };
            Grid.SetColumn(valText, 1);
            headerGrid.Children.Add(valText);
            sp.Children.Add(headerGrid);

            var slider = new Slider
            {
                Minimum = min,
                Maximum = max,
                SmallChange = tick,
                LargeChange = tick * 2,
                Value = value,
                Margin = new Thickness(0, 10, 0, 0),
                Cursor = Cursors.Hand
            };
            slider.ValueChanged += (s, e) =>
            {
                valText.Text = slider.Value.ToString("0.##");
                onChanged(slider.Value);
            };
            sp.Children.Add(slider);

            card.Child = sp;
            ContentStack.Children.Add(card);
        }

        private void AddInfoCard(string label, string value)
        {
            var card = new Border
            {
                Background = (Brush)new BrushConverter().ConvertFrom("#121215")!,
                BorderBrush = (Brush)new BrushConverter().ConvertFrom("#1FFFFFFF")!,
                BorderThickness = new Thickness(1),
                CornerRadius = new CornerRadius(10),
                Padding = new Thickness(16, 12, 16, 12),
                Margin = new Thickness(0, 0, 0, 10)
            };

            var grid = new Grid();
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

            var lbl = new TextBlock { Text = label, Foreground = (Brush)new BrushConverter().ConvertFrom("#88FFFFFF")!, FontSize = 13, VerticalAlignment = VerticalAlignment.Center };
            Grid.SetColumn(lbl, 0);
            grid.Children.Add(lbl);

            var val = new TextBlock { Text = value, Foreground = Brushes.White, FontSize = 13, FontWeight = FontWeights.SemiBold, VerticalAlignment = VerticalAlignment.Center };
            Grid.SetColumn(val, 1);
            grid.Children.Add(val);

            card.Child = grid;
            ContentStack.Children.Add(card);
        }

        private void AddButton(string text, string description, Action onClick)
        {
            var card = new Border
            {
                Background = (Brush)new BrushConverter().ConvertFrom("#121215")!,
                BorderBrush = (Brush)new BrushConverter().ConvertFrom("#1FFFFFFF")!,
                BorderThickness = new Thickness(1),
                CornerRadius = new CornerRadius(10),
                Padding = new Thickness(16, 12, 16, 12),
                Margin = new Thickness(0, 0, 0, 10)
            };

            var grid = new Grid();
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

            var sp = new StackPanel { VerticalAlignment = VerticalAlignment.Center };
            sp.Children.Add(new TextBlock { Text = text, Foreground = Brushes.White, FontSize = 13.5, FontWeight = FontWeights.SemiBold });
            sp.Children.Add(new TextBlock { Text = description, Foreground = (Brush)new BrushConverter().ConvertFrom("#88FFFFFF")!, FontSize = 12, Margin = new Thickness(0, 2, 0, 0) });
            Grid.SetColumn(sp, 0);
            grid.Children.Add(sp);

            var btn = new Button
            {
                Content = "Open",
                Padding = new Thickness(16, 6, 16, 6),
                Background = (Brush)new BrushConverter().ConvertFrom("#25FFFFFF")!,
                Foreground = Brushes.White,
                BorderThickness = new Thickness(0),
                Cursor = Cursors.Hand
            };
            btn.Click += (s, e) => onClick();
            Grid.SetColumn(btn, 1);
            grid.Children.Add(btn);

            card.Child = grid;
            ContentStack.Children.Add(card);
        }
    }
}
