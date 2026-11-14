using System;
using System.IO;
using System.Windows;

namespace NotchIsland
{
    public partial class App : Application
    {
        public static void Log(string msg)
        {
            try
            {
                File.AppendAllText(@"c:\project\rea\app_startup.log", $"[{DateTime.Now:HH:mm:ss.fff}] {msg}\r\n");
            }
            catch { }
        }

        public App()
        {
            Log("App ctor start");
            AppDomain.CurrentDomain.UnhandledException += (s, e) =>
            {
                Log($"Unhandled: {e.ExceptionObject}");
            };

            DispatcherUnhandledException += (s, e) =>
            {
                Log($"Dispatcher: {e.Exception}");
                e.Handled = true;
            };

            Exit += (s, e) =>
            {
                Log($"App.Exit: Code={e.ApplicationExitCode}");
            };
            Log("App ctor end");
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            Log("App OnStartup start");
            base.OnStartup(e);
            Log("App OnStartup end");
        }
    }
}
