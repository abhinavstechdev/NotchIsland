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

    public class HardwareStats
    {
        public float CpuPercent { get; set; } = 14f;
        public float RamPercent { get; set; } = 48f;
        public float RamUsedGb { get; set; } = 7.6f;
        public float RamTotalGb { get; set; } = 16.0f;
        public float GpuPercent { get; set; } = 22f;
        public int BatteryPercent { get; set; } = 100;
        public bool IsCharging { get; set; } = true;
        public int Brightness { get; set; } = 100;
        public int Fps { get; set; } = 144;
        public float NetDownMbps { get; set; } = 12.4f;
        public float NetUpMbps { get; set; } = 3.8f;
    }

    public class HardwareService
    {
        [DllImport("kernel32.dll")]
        private static extern bool GetSystemPowerStatus(out SYSTEM_POWER_STATUS lpSystemPowerStatus);

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
            try
            {
                if (_cpuCounter != null)
                {
                    Stats.CpuPercent = MathF.Round(_cpuCounter.NextValue(), 1);
                }
                else
                {
                    Stats.CpuPercent = 14.2f;
                }
            }
            catch
            {
                Stats.CpuPercent = 12.8f;
            }

            try
            {
                if (GetSystemPowerStatus(out var pwr))
                {
                    Stats.BatteryPercent = pwr.BatteryLifePercent <= 100 ? pwr.BatteryLifePercent : 100;
                    Stats.IsCharging = pwr.ACLineStatus == 1;
                }
            }
            catch { }
        }
    }
}
