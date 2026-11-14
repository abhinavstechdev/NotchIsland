using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;

namespace NotchIsland.Services
{
    public class IslandSettings
    {
        // 1. General
        public bool SlimWhenCovered { get; set; } = true;
        public bool HideInFullscreen { get; set; } = false;
        public bool HideFromCaptures { get; set; } = false;
        public int HoverDelayMs { get; set; } = 150;
        public int TopGap { get; set; } = 6;
        public int TopGapDip { get; set; } = 6;
        public int EdgeMargin { get; set; } = 24;
        public int SidePosition { get; set; } = 50;
        public bool HiddenUntilHover { get; set; } = false;

        // 2. Accessibility
        public bool HighContrastText { get; set; } = false;
        public bool ReduceMotion { get; set; } = false;
        public double FontScale { get; set; } = 1.0;
        public bool ScreenReaderAnnounce { get; set; } = false;

        // 3. Appearance
        public string Surface { get; set; } = "black"; // black, glass, clear
        public string Accent { get; set; } = "white";
        public int GlassBlur { get; set; } = 50;
        public int CornerRoundness { get; set; } = 100;
        public int Size { get; set; } = 100;
        public int OpenSize { get; set; } = 100;
        public string Motion { get; set; } = "balanced"; // balanced, snappy, fluid
        public string AnimationSpeed { get; set; } = "normal";
        public string Visualizer { get; set; } = "pixels";
        public string ProgressBar { get; set; } = "wave";
        public string MusicBackground { get; set; } = "album";

        // 4. Island
        public string Position { get; set; } = "Center"; // Center, Left, Right
        public string Shape { get; set; } = "Island"; // Island, Notch
        public string IdleShows { get; set; } = "empty"; // empty, time, weather, pet
        public string IdlePet { get; set; } = "cat"; // cat, dog, bunny
        public bool NotchEarFlare { get; set; } = true;

        // 5. Taskbar
        public bool TaskbarIslandIntegration { get; set; } = true;
        public bool AutoPositionOverTaskbar { get; set; } = false;
        public bool TaskbarHoverExpand { get; set; } = true;

        // 6. Panels
        public bool MediaPanel { get; set; } = true;
        public bool ShelfEnabled { get; set; } = true;
        public bool CapturesToShelf { get; set; } = true;
        public bool ClipboardEnabled { get; set; } = true;
        public int ClipboardHistorySize { get; set; } = 12;
        public int ClipboardKeepHours { get; set; } = 10;
        public bool ControlsPanel { get; set; } = true;
        public bool WebPanel { get; set; } = true;
        public bool NotificationsPanel { get; set; } = true;
        public bool LauncherPanel { get; set; } = true;
        public bool DeckPanel { get; set; } = true;
        public int DeckPort { get; set; } = 8088;
        public string LastPanel { get; set; } = "Call";

        // 7. Widgets
        public bool ShowDateWidget { get; set; } = true;
        public bool ShowTimerWidget { get; set; } = true;
        public bool ShowWeatherWidget { get; set; } = true;
        public bool ShowStatsWidget { get; set; } = true;
        public int FocusMinutes { get; set; } = 25;
        public int BreakMinutes { get; set; } = 5;

        // 8. Calls
        public bool AutoDetectCalls { get; set; } = true;
        public bool MicMuteShortcutEnabled { get; set; } = true;
        public string PreferredAudioInput { get; set; } = "Default";
        public string PreferredAudioOutput { get; set; } = "Default";

        // 9. Streaming
        public bool ObsIntegration { get; set; } = true;
        public bool LiveIndicator { get; set; } = true;
        public bool StreamingMuteStatus { get; set; } = true;

        // 10. Gaming
        public bool GameDetection { get; set; } = true;
        public bool SlimGamingHud { get; set; } = true;
        public bool ShowFpsCounter { get; set; } = false;

        // 11. AI Agents
        public bool ClaudeAgentEnabled { get; set; } = true;
        public bool ChatGptAgentEnabled { get; set; } = true;
        public bool ShowAgentLiveActivity { get; set; } = true;

        // 12. Tools
        public bool ReplaceVolumeFlyout { get; set; } = true;
        public bool ShowVolumeHud { get; set; } = true;
        public bool ShowBrightnessHud { get; set; } = true;
        public bool ShowBatteryHud { get; set; } = true;
        public bool ShowLockKeysHud { get; set; } = true;
        public bool MirrorNotifications { get; set; } = true;
        public int NotificationSeconds { get; set; } = 5;

        // 13. Keys
        public string HotkeyToggleCard { get; set; } = "Ctrl+Alt+N";
        public string HotkeyPlayPause { get; set; } = "Ctrl+Alt+P";
        public string HotkeyTimer { get; set; } = "Ctrl+Alt+T";
        public string HotkeyHide { get; set; } = "Ctrl+Alt+H";
        public string HotkeyMicMute { get; set; } = "Ctrl+Alt+Shift+M";
        public string HotkeyScreenshot { get; set; } = "Ctrl+Alt+S";
        public string HotkeyScreenRecord { get; set; } = "Ctrl+Alt+Shift+R";

        // 14. Web Pins
        public List<string> WebPins { get; set; } = new()
        {
            "https://www.google.com",
            "https://www.youtube.com",
            "https://github.com",
            "https://chatgpt.com",
            "https://reddit.com"
        };
    }

    public static class SettingsService
    {
        private static readonly string SettingsDir = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "NotchIsland");
        private static readonly string SettingsFile = Path.Combine(SettingsDir, "settings.json");

        private static IslandSettings? _current;
        private static readonly JsonSerializerOptions JsonOpts = new() { WriteIndented = true };

        public static event Action? OnSettingsChanged;

        public static IslandSettings Current
        {
            get
            {
                if (_current == null) Load();
                return _current!;
            }
        }

        public static void Load()
        {
            try
            {
                if (File.Exists(SettingsFile))
                {
                    string json = File.ReadAllText(SettingsFile);
                    _current = JsonSerializer.Deserialize<IslandSettings>(json, JsonOpts);
                }
            }
            catch { }

            _current ??= new IslandSettings();
        }

        public static void Save()
        {
            try
            {
                Directory.CreateDirectory(SettingsDir);
                string json = JsonSerializer.Serialize(Current, JsonOpts);
                File.WriteAllText(SettingsFile, json);
                OnSettingsChanged?.Invoke();
            }
            catch { }
        }
    }
}
