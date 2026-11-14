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
                    AddHeader("General Settings", "Basic island placement and behavior");
                    AddToggle("Slim when covered", "Island shrinks to slim line when active window maximizes", s.SlimWhenCovered, v => { s.SlimWhenCovered = v; Save(); });
                    AddToggle("Hide in fullscreen", "Automatically hide the island in games or fullscreen video", s.HideInFullscreen, v => { s.HideInFullscreen = v; Save(); });
                    AddToggle("Hide from screen captures", "Exclude island from screenshots and OBS recording", s.HideFromCaptures, v => { s.HideFromCaptures = v; Save(); });
                    AddSlider("Hover Delay (ms)", "Time required before island expands on hover", s.HoverDelayMs, 0, 500, 10, v => { s.HoverDelayMs = (int)v; Save(); });
                    AddSlider("Top Gap (pixels)", "Distance from the top edge of screen", s.TopGap, 0, 30, 1, v => { s.TopGap = (int)v; Save(); });
                    break;

                case "Appearance":
                    AddHeader("Appearance", "Island look, styling, and continuous squircle curvature");
                    AddToggle("OLED Deep Black", "Zero-backlight pure OLED black (#000000)", s.Surface == "black", v => { s.Surface = v ? "black" : "glass"; Save(); });
                    AddSlider("Corner Curvature", "Squircle roundness percentage", s.CornerRoundness, 50, 100, 5, v => { s.CornerRoundness = (int)v; Save(); });
                    AddSlider("Glass Blur Strength", "Backdrop acrylic / frosted blur intensity", s.GlassBlur, 10, 100, 5, v => { s.GlassBlur = (int)v; Save(); });
                    AddSlider("Island Scale", "Overall UI scaling percentage", s.Size, 80, 150, 5, v => { s.Size = (int)v; Save(); });
                    break;

                case "Media":
                    AddHeader("Media & Music", "Playback controls, visualizers, and album art");
                    AddToggle("Album Art Backdrop", "Vibrant blurred artwork behind expanded music card", s.MusicBackground == "album", v => { s.MusicBackground = v ? "album" : "dark"; Save(); });
                    AddToggle("Audio Visualizer", "Live spectrum equalizer bars on compact pill", s.Visualizer == "pixels", v => { s.Visualizer = v ? "pixels" : "none"; Save(); });
                    AddToggle("Wave Progress Bar", "Fluid continuous wave animation on scrub slider", s.ProgressBar == "wave", v => { s.ProgressBar = v ? "wave" : "flat"; Save(); });
                    break;

                case "Shelf":
                    AddHeader("Shelf & Clipboard", "Quick drop file shelf and historical snippet manager");
                    AddToggle("Enable File Shelf", "Allow dropping files onto the island to hold and share", s.ShelfEnabled, v => { s.ShelfEnabled = v; Save(); });
                    AddToggle("Enable Clipboard History", "Monitor clipboard and preserve recent text and images", s.ClipboardEnabled, v => { s.ClipboardEnabled = v; Save(); });
                    AddSlider("Clipboard History Limit", "Maximum number of items preserved", s.ClipboardHistorySize, 5, 50, 5, v => { s.ClipboardHistorySize = (int)v; Save(); });
                    AddButton("Clear All Shelf Files", "Delete all temporary files stored on shelf", () => FileShelfService.ClearAll());
                    break;

                case "Widgets":
                    AddHeader("Widgets & Mini Apps", "Interactive desktop widgets and idle island faces");
                    AddToggle("Show Date & Clock Widget", "Digital clock with live time and date", s.ShowDateWidget, v => { s.ShowDateWidget = v; Save(); });
                    AddToggle("Show Stopwatch Widget", "Quick timer and countdown tool", s.ShowTimerWidget, v => { s.ShowTimerWidget = v; Save(); });
                    AddToggle("Show Weather Widget", "Current weather and temperature", s.ShowWeatherWidget, v => { s.ShowWeatherWidget = v; Save(); });
                    AddToggle("Show Hardware Stats Widget", "Live CPU, RAM, and battery monitor", s.ShowStatsWidget, v => { s.ShowStatsWidget = v; Save(); });
                    break;

                case "Controls":
                    AddHeader("Controls & Windows HUDs", "Hardware flyout replacements and native system toggles");
                    AddToggle("Replace Windows Volume Flyout", "Show sleek top island HUD instead of bulky Windows box", s.ReplaceVolumeFlyout, v => { s.ReplaceVolumeFlyout = v; Save(); });
                    AddToggle("Show Brightness HUD", "Show island pill when monitor brightness changes", s.ShowBrightnessHud, v => { s.ShowBrightnessHud = v; Save(); });
                    AddToggle("Show Battery & Charging HUD", "Show notification when charger is connected", s.ShowBatteryHud, v => { s.ShowBatteryHud = v; Save(); });
                    AddToggle("Show Lock Keys HUD", "Show Caps Lock / Num Lock alerts", s.ShowLockKeysHud, v => { s.ShowLockKeysHud = v; Save(); });
                    break;

                case "Hotkeys":
                    AddHeader("Global Hotkeys", "Keyboard shortcuts for lightning-fast control");
                    AddInfoCard("Toggle Island Card", "Ctrl + Alt + N");
                    AddInfoCard("Play / Pause Media", "Ctrl + Alt + P");
                    AddInfoCard("Mute Microphone", "Ctrl + Alt + M");
                    AddInfoCard("Quick Screenshot", "Ctrl + Alt + S");
                    AddInfoCard("Screen Snip Tool", "Ctrl + Alt + Shift + S");
                    AddInfoCard("Stopwatch / Timer", "Ctrl + Alt + T");
                    break;

                case "Deck":
                    AddHeader("PC Deck Companion", "Control this PC from any phone or tablet browser");
                    AddToggle("PC Deck Server Enabled", "Run local web server on port 8088", s.DeckPanel, v => { s.DeckPanel = v; Save(); });
                    AddButton("Open PC Deck in Browser", "Launch http://localhost:8088 in your default browser", () =>
                    {
                        Process.Start(new ProcessStartInfo("http://localhost:8088") { UseShellExecute = true });
                    });
                    break;

                case "About":
                    AddHeader("About NotchIsland", "100% Free & Open Source Native Windows Dynamic Island");
                    AddInfoCard("Version", "0.5.0 (Single-file Native Executable)");
                    AddInfoCard("Architecture", "C# .NET 9 + WPF + SkiaSharp + DirectComposition");
                    AddInfoCard("Target", "Native Windows 10 & 11 (x64)");
                    AddButton("Visit GitHub Repository", "Open https://github.com/abhinavstechdev/NotchIsland", () =>
                    {
                        Process.Start(new ProcessStartInfo("https://github.com/abhinavstechdev/NotchIsland") { UseShellExecute = true });
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

            var titleSp = new StackPanel();
            titleSp.Children.Add(new TextBlock { Text = title, Foreground = Brushes.White, FontSize = 13.5, FontWeight = FontWeights.SemiBold });
            titleSp.Children.Add(new TextBlock { Text = description, Foreground = (Brush)new BrushConverter().ConvertFrom("#88FFFFFF")!, FontSize = 12, Margin = new Thickness(0, 2, 0, 0) });
            Grid.SetColumn(titleSp, 0);
            headerGrid.Children.Add(titleSp);

            var valTb = new TextBlock
            {
                Text = value.ToString("0"),
                Foreground = (Brush)new BrushConverter().ConvertFrom("#00F2FE")!,
                FontSize = 13.5,
                FontWeight = FontWeights.Bold,
                VerticalAlignment = VerticalAlignment.Center
            };
            Grid.SetColumn(valTb, 1);
            headerGrid.Children.Add(valTb);
            sp.Children.Add(headerGrid);

            var slider = new Slider
            {
                Minimum = min,
                Maximum = max,
                Value = value,
                TickFrequency = tick,
                IsSnapToTickEnabled = true,
                Margin = new Thickness(0, 10, 0, 0),
                Cursor = Cursors.Hand
            };
            slider.ValueChanged += (s, e) =>
            {
                valTb.Text = slider.Value.ToString("0");
                onChanged(slider.Value);
            };
            sp.Children.Add(slider);

            card.Child = sp;
            ContentStack.Children.Add(card);
        }

        private void AddButton(string title, string description, Action onClick)
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

            var btn = new Button
            {
                Content = "Action",
                Padding = new Thickness(14, 6, 14, 6),
                Background = (Brush)new BrushConverter().ConvertFrom("#20FFFFFF")!,
                Foreground = Brushes.White,
                Cursor = Cursors.Hand
            };
            btn.Click += (s, e) => onClick();
            Grid.SetColumn(btn, 1);
            grid.Children.Add(btn);

            card.Child = grid;
            ContentStack.Children.Add(card);
        }

        private void AddInfoCard(string title, string value)
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

            var titleTb = new TextBlock { Text = title, Foreground = (Brush)new BrushConverter().ConvertFrom("#88FFFFFF")!, FontSize = 13, VerticalAlignment = VerticalAlignment.Center };
            Grid.SetColumn(titleTb, 0);
            grid.Children.Add(titleTb);

            var valTb = new TextBlock { Text = value, Foreground = Brushes.White, FontSize = 13, FontWeight = FontWeights.SemiBold, VerticalAlignment = VerticalAlignment.Center };
            Grid.SetColumn(valTb, 1);
            grid.Children.Add(valTb);

            card.Child = grid;
            ContentStack.Children.Add(card);
        }
    }
}
