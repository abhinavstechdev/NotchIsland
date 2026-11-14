using System;
using System.Diagnostics;
using System.IO;
using System.Management;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media.Imaging;

namespace NotchIsland.Services
{
    public static class SystemControlService
    {
        #region CoreAudio COM Interfaces
        [ComImport]
        [Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
        private class MMDeviceEnumeratorComObject { }

        [Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        private interface IMMDeviceEnumerator
        {
            int EnumAudioEndpoints(int dataFlow, int stateMask, out nint devices);
            int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice endpoint);
            int GetDevice(string id, out IMMDevice device);
            int RegisterEndpointNotificationCallback(nint client);
            int UnregisterEndpointNotificationCallback(nint client);
        }

        [Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        private interface IMMDevice
        {
            int Activate(ref Guid id, int clsCtx, nint activationParams, [MarshalAs(UnmanagedType.IUnknown)] out object interfacePointer);
            int OpenPropertyStore(int stgmAccess, out nint properties);
            int GetId([MarshalAs(UnmanagedType.LPWStr)] out string id);
            int GetState(out int state);
        }

        [Guid("5CDF2C82-841E-4546-9722-0CF74078229A"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        private interface IAudioEndpointVolume
        {
            int RegisterControlChangeNotify(nint client);
            int UnregisterControlChangeNotify(nint client);
            int GetChannelCount(out int channelCount);
            int SetMasterVolumeLevel(float levelDb, ref Guid eventContext);
            int SetMasterVolumeLevelScalar(float level, ref Guid eventContext);
            int GetMasterVolumeLevel(out float levelDb);
            int GetMasterVolumeLevelScalar(out float level);
            int SetChannelVolumeLevel(uint channelNumber, float levelDb, ref Guid eventContext);
            int SetChannelVolumeLevelScalar(uint channelNumber, float level, ref Guid eventContext);
            int GetChannelVolumeLevel(uint channelNumber, out float levelDb);
            int GetChannelVolumeLevelScalar(uint channelNumber, out float level);
            int SetMute([MarshalAs(UnmanagedType.Bool)] bool isMuted, ref Guid eventContext);
            int GetMute([MarshalAs(UnmanagedType.Bool)] out bool isMuted);
            int GetVolumeStepInfo(out uint step, out uint stepCount);
            int VolumeStepUp(ref Guid eventContext);
            int VolumeStepDown(ref Guid eventContext);
            int QueryHardwareSupport(out uint hardwareSupportMask);
            int GetVolumeRange(out float volumeMin, out float volumeMax, out float volumeStep);
        }

        private static IAudioEndpointVolume? GetEndpointVolume()
        {
            try
            {
                var enumerator = (IMMDeviceEnumerator)new MMDeviceEnumeratorComObject();
                enumerator.GetDefaultAudioEndpoint(0, 1, out IMMDevice device);
                var iid = typeof(IAudioEndpointVolume).GUID;
                device.Activate(ref iid, 0x17, 0, out object endpoint);
                return (IAudioEndpointVolume)endpoint;
            }
            catch
            {
                return null;
            }
        }
        #endregion

        #region Volume Control
        public static float GetMasterVolume()
        {
            try
            {
                var vol = GetEndpointVolume();
                if (vol != null)
                {
                    vol.GetMasterVolumeLevelScalar(out float level);
                    return level * 100f;
                }
            }
            catch { }
            return 50f;
        }

        public static void SetMasterVolume(float levelPercent)
        {
            try
            {
                var vol = GetEndpointVolume();
                if (vol != null)
                {
                    float scalar = Math.Clamp(levelPercent / 100f, 0f, 1f);
                    Guid empty = Guid.Empty;
                    vol.SetMasterVolumeLevelScalar(scalar, ref empty);
                }
            }
            catch { }
        }

        public static bool GetIsMuted()
        {
            try
            {
                var vol = GetEndpointVolume();
                if (vol != null)
                {
                    vol.GetMute(out bool muted);
                    return muted;
                }
            }
            catch { }
            return false;
        }

        public static void SetIsMuted(bool isMuted)
        {
            try
            {
                var vol = GetEndpointVolume();
                if (vol != null)
                {
                    Guid empty = Guid.Empty;
                    vol.SetMute(isMuted, ref empty);
                }
            }
            catch { }
        }

        public static void ToggleMute()
        {
            SetIsMuted(!GetIsMuted());
        }
        #endregion

        #region Brightness Control (WMI)
        public static int GetBrightness()
        {
            try
            {
                using var searcher = new ManagementObjectSearcher(@"root\wmi", "SELECT CurrentBrightness FROM WmiMonitorBrightness");
                foreach (ManagementObject mo in searcher.Get())
                {
                    return Convert.ToInt32(mo["CurrentBrightness"]);
                }
            }
            catch { }
            return 70;
        }

        public static void SetBrightness(int brightnessPercent)
        {
            try
            {
                brightnessPercent = Math.Clamp(brightnessPercent, 0, 100);
                using var searcher = new ManagementObjectSearcher(@"root\wmi", "SELECT * FROM WmiMonitorBrightnessMethods");
                foreach (ManagementObject mo in searcher.Get())
                {
                    mo.InvokeMethod("WmiSetBrightness", new object[] { uint.MaxValue, (byte)brightnessPercent });
                }
            }
            catch { }
        }
        #endregion

        #region Native Screenshot & Snip
        [DllImport("user32.dll")]
        private static extern nint GetDesktopWindow();

        [DllImport("user32.dll")]
        private static extern nint GetWindowDC(nint hWnd);

        [DllImport("user32.dll")]
        private static extern int ReleaseDC(nint hWnd, nint hDC);

        [DllImport("gdi32.dll")]
        private static extern bool BitBlt(nint hObject, int nXDest, int nYDest, int nWidth, int nHeight, nint hObjectSource, int nXSrc, int nYSrc, int dwRop);

        [DllImport("gdi32.dll")]
        private static extern nint CreateCompatibleBitmap(nint hDC, int nWidth, int nHeight);

        [DllImport("gdi32.dll")]
        private static extern nint CreateCompatibleDC(nint hDC);

        [DllImport("gdi32.dll")]
        private static extern bool DeleteDC(nint hDC);

        [DllImport("gdi32.dll")]
        private static extern bool DeleteObject(nint hObject);

        [DllImport("gdi32.dll")]
        private static extern nint SelectObject(nint hDC, nint hObject);

        private const int SRCCOPY = 0x00CC0020;

        public static string? TakeScreenshot()
        {
            try
            {
                int screenWidth = (int)SystemParameters.PrimaryScreenWidth;
                int screenHeight = (int)SystemParameters.PrimaryScreenHeight;

                nint hDesktopWnd = GetDesktopWindow();
                nint hDesktopDC = GetWindowDC(hDesktopWnd);
                nint hMemoryDC = CreateCompatibleDC(hDesktopDC);
                nint hBitmap = CreateCompatibleBitmap(hDesktopDC, screenWidth, screenHeight);
                nint hOldBitmap = SelectObject(hMemoryDC, hBitmap);

                BitBlt(hMemoryDC, 0, 0, screenWidth, screenHeight, hDesktopDC, 0, 0, SRCCOPY);

                var bitmapSource = Imaging.CreateBitmapSourceFromHBitmap(
                    hBitmap, 0, Int32Rect.Empty, BitmapSizeOptions.FromEmptyOptions());

                SelectObject(hMemoryDC, hOldBitmap);
                DeleteObject(hBitmap);
                DeleteDC(hMemoryDC);
                ReleaseDC(hDesktopWnd, hDesktopDC);

                string screenshotsFolder = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.MyPictures), "Screenshots");
                Directory.CreateDirectory(screenshotsFolder);

                string filename = $"Screenshot_{DateTime.Now:yyyyMMdd_HHmmss}.png";
                string fullPath = Path.Combine(screenshotsFolder, filename);

                using (var fileStream = new FileStream(fullPath, FileMode.Create))
                {
                    var encoder = new PngBitmapEncoder();
                    encoder.Frames.Add(BitmapFrame.Create(bitmapSource));
                    encoder.Save(fileStream);
                }

                // Copy to clipboard
                Clipboard.SetImage(bitmapSource);

                // Add to shelf
                FileShelfService.AddFile(fullPath);

                return fullPath;
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"TakeScreenshot error: {ex.Message}");
                return null;
            }
        }

        public static void LaunchSnip()
        {
            try
            {
                Process.Start(new ProcessStartInfo("ms-screenclip:") { UseShellExecute = true });
            }
            catch { }
        }
        #endregion

        #region Color Picker
        [DllImport("user32.dll")]
        private static extern bool GetCursorPos(out POINT lpPoint);

        [DllImport("user32.dll")]
        private static extern nint GetDC(nint hWnd);

        [DllImport("gdi32.dll")]
        private static extern uint GetPixel(nint hdc, int nXPos, int nYPos);

        [StructLayout(LayoutKind.Sequential)]
        private struct POINT { public int X; public int Y; }

        public static string PickColorUnderCursor()
        {
            try
            {
                if (GetCursorPos(out POINT pt))
                {
                    nint hdc = GetDC(0);
                    uint pixel = GetPixel(hdc, pt.X, pt.Y);
                    ReleaseDC(0, hdc);

                    byte r = (byte)(pixel & 0x000000FF);
                    byte g = (byte)((pixel & 0x0000FF00) >> 8);
                    byte b = (byte)((pixel & 0x00FF0000) >> 16);

                    string hex = $"#{r:X2}{g:X2}{b:X2}";
                    Clipboard.SetText(hex);
                    return hex;
                }
            }
            catch { }
            return "#000000";
        }
        #endregion

        #region Windows Settings Launchers
        public static void OpenBluetoothSettings()
        {
            try { Process.Start(new ProcessStartInfo("ms-settings:bluetooth") { UseShellExecute = true }); } catch { }
        }

        public static void OpenNetworkSettings()
        {
            try { Process.Start(new ProcessStartInfo("ms-settings:network") { UseShellExecute = true }); } catch { }
        }

        public static void OpenNightLightSettings()
        {
            try { Process.Start(new ProcessStartInfo("ms-settings:nightlight") { UseShellExecute = true }); } catch { }
        }
        #endregion
    }
}
