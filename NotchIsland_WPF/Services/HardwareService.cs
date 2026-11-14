using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace NotchIsland.Services
{
    [StructLayout(LayoutKind.Sequential)]
    public struct SYSTEM_POWER_STATUS
    {
        public byte ACLineStatus;
        public byte BatteryFlag;
        public byte BatteryLifePercent;
        public byte SystemStatusFlag;
        public int BatteryLifeTime;
        public int BatteryFullLifeTime;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Auto)]
    public class MEMORYSTATUSEX
    {
        public uint dwLength;
        public uint dwMemoryLoad;
        public ulong ullTotalPhys;
        public ulong ullAvailPhys;
        public ulong ullTotalPageFile;
        public ulong ullAvailPageFile;
        public ulong ullTotalVirtual;
        public ulong ullAvailVirtual;
        public ulong ullAvailExtendedVirtual;

        public MEMORYSTATUSEX()
        {
            dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX));
        }
    }

    public class HardwareStats
    {
        public float CpuPercent { get; set; } = 14f;
        public float RamPercent { get; set; } = 48f;
        public float RamUsedGb { get; set; } = 7.6f;
        public float RamTotalGb { get; set; } = 16.0f;
        public float GpuPercent { get; set; } = 22f;
        public int BatteryPercent { get; set; } = 100;
        public bool IsCharging { get; set; } = true;
        public int Brightness { get; set; } = 70;
        public int Volume { get; set; } = 50;
        public int Fps { get; set; } = 144;
        public float NetDownMbps { get; set; } = 12.4f;
        public float NetUpMbps { get; set; } = 3.8f;
    }

    public class HardwareService
    {
        [DllImport("kernel32.dll")]
        private static extern bool GetSystemPowerStatus(out SYSTEM_POWER_STATUS lpSystemPowerStatus);

        [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GlobalMemoryStatusEx([In, Out] MEMORYSTATUSEX lpBuffer);

        public HardwareStats Stats { get; } = new HardwareStats();
        private PerformanceCounter? _cpuCounter;

        public HardwareService()
        {
            try
            {
                _cpuCounter = new PerformanceCounter("Processor", "% Processor Time", "_Total");
                _cpuCounter.NextValue();
            }
            catch { }
        }

        public void Update()
        {
            // CPU
            try
            {
                if (_cpuCounter != null)
                {
                    Stats.CpuPercent = MathF.Round(_cpuCounter.NextValue(), 1);
                }
            }
            catch { }

            // RAM
            try
            {
                var mem = new MEMORYSTATUSEX();
                if (GlobalMemoryStatusEx(mem))
                {
                    Stats.RamPercent = mem.dwMemoryLoad;
                    double total = mem.ullTotalPhys / (1024.0 * 1024.0 * 1024.0);
                    double used = (mem.ullTotalPhys - mem.ullAvailPhys) / (1024.0 * 1024.0 * 1024.0);
                    Stats.RamTotalGb = MathF.Round((float)total, 1);
                    Stats.RamUsedGb = MathF.Round((float)used, 1);
                }
            }
            catch { }

            // Battery
            try
            {
                if (GetSystemPowerStatus(out var pwr))
                {
                    Stats.BatteryPercent = pwr.BatteryLifePercent <= 100 ? pwr.BatteryLifePercent : 100;
                    Stats.IsCharging = pwr.ACLineStatus == 1;
                }
            }
            catch { }

            // Volume & Brightness
            try
            {
                Stats.Volume = (int)SystemControlService.GetMasterVolume();
                Stats.Brightness = SystemControlService.GetBrightness();
            }
            catch { }
        }
    }
}
