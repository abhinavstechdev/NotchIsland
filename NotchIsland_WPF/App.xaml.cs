using System;
using System.IO;
using System.Windows;

namespace NotchIsland
{
    public partial class App : Application
    {
        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += (s, e) =>
            {
                File.WriteAllText(@"c:\project\rea\crash.log", $"Unhandled: {e.ExceptionObject}");
            };

            DispatcherUnhandledException += (s, e) =>
            {
                File.WriteAllText(@"c:\project\rea\crash.log", $"Dispatcher: {e.Exception}");
                e.Handled = true;
            };

            Exit += (s, e) =>
            {
                File.AppendAllText(@"c:\project\rea\crash.log", $"\nApp.Exit: Code={e.ApplicationExitCode}\nStack:\n{Environment.StackTrace}");
            };
        }
    }
}
