using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace NotchIsland.Services
{
    public class IslandSettings
    {
        public bool SlimWhenCovered { get; set; } = true;
        public bool HideInFullscreen { get; set; } = false;
        public bool HideFromCaptures { get; set; } = false;
        public bool ReplaceVolumeFlyout { get; set; } = true;
        public bool ShowVolumeHud { get; set; } = true;
        public bool ShowBrightnessHud { get; set; } = true;
        public bool ShowBatteryHud { get; set; } = true;
        public bool ShowBluetoothHud { get; set; } = true;
        public bool ShowLockKeysHud { get; set; } = true;
        public bool MirrorNotifications { get; set; } = true;
        public int NotificationSeconds { get; set; } = 5;

        public string Surface { get; set; } = "black"; // black, glass, clear
        public string Accent { get; set; } = "white";
        public int GlassBlur { get; set; } = 50;
        public string Position { get; set; } = "Center"; // Center, Left, Right
        public string Shape { get; set; } = "Island"; // Island, Notch
        public int CornerRoundness { get; set; } = 100;
        public int Size { get; set; } = 100;
        public int OpenSize { get; set; } = 100;
        public string Motion { get; set; } = "balanced";
        public string Visualizer { get; set; } = "pixels";
        public string MusicBackground { get; set; } = "album";
        public string ProgressBar { get; set; } = "wave";
        public string AnimationSpeed { get; set; } = "normal";

        public int TopGap { get; set; } = 6;
        public int EdgeMargin { get; set; } = 24;
        public int HoverDelayMs { get; set; } = 150;
        public bool HiddenUntilHover { get; set; } = false;

        public string IdleShows { get; set; } = "empty"; // empty, time, weather, pet
        public string IdlePet { get; set; } = "cat"; // cat, dog, bunny

        public bool ShelfEnabled { get; set; } = true;
        public bool ClipboardEnabled { get; set; } = true;
        public int ClipboardHistorySize { get; set; } = 20;

        public bool ShowDateWidget { get; set; } = true;
        public bool ShowTimerWidget { get; set; } = true;
        public bool ShowWeatherWidget { get; set; } = true;
        public bool ShowStatsWidget { get; set; } = true;

        public bool ControlsPanel { get; set; } = true;
        public bool WebPanel { get; set; } = true;
        public bool NotificationsPanel { get; set; } = true;
        public bool LauncherPanel { get; set; } = true;
        public bool DeckPanel { get; set; } = true;
        public int DeckPort { get; set; } = 8088;

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
