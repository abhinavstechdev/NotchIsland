using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;

namespace NotchIsland.Services
{
    public class ShelfItem
    {
        public string FileName { get; set; } = "";
        public string FilePath { get; set; } = "";
        public string FileSizeFormatted { get; set; } = "";
        public string Extension { get; set; } = "";
        public DateTime AddedAt { get; set; } = DateTime.Now;
    }

    public static class FileShelfService
    {
        private static readonly string ShelfDir = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "NotchIsland", "Shelf");

        public static ObservableCollection<ShelfItem> Items { get; } = new();

        static FileShelfService()
        {
            try
            {
                Directory.CreateDirectory(ShelfDir);
                LoadExisting();
            }
            catch { }
        }

        private static void LoadExisting()
        {
            try
            {
                var files = Directory.GetFiles(ShelfDir);
                foreach (var f in files)
                {
                    var fi = new FileInfo(f);
                    Items.Add(new ShelfItem
                    {
                        FileName = fi.Name,
                        FilePath = fi.FullName,
                        FileSizeFormatted = FormatBytes(fi.Length),
                        Extension = fi.Extension.ToUpperInvariant(),
                        AddedAt = fi.CreationTime
                    });
                }
            }
            catch { }
        }

        public static void AddFile(string sourcePath)
        {
            try
            {
                if (!File.Exists(sourcePath)) return;
                var fi = new FileInfo(sourcePath);
                string destPath = Path.Combine(ShelfDir, fi.Name);

                // If not already in shelf dir, copy it
                if (!sourcePath.Equals(destPath, StringComparison.OrdinalIgnoreCase))
                {
                    File.Copy(sourcePath, destPath, true);
                }

                var item = new ShelfItem
                {
                    FileName = fi.Name,
                    FilePath = destPath,
                    FileSizeFormatted = FormatBytes(fi.Length),
                    Extension = fi.Extension.ToUpperInvariant(),
                    AddedAt = DateTime.Now
                };

                // Add to start of collection
                System.Windows.Application.Current?.Dispatcher.Invoke(() =>
                {
                    Items.Insert(0, item);
                });
            }
            catch { }
        }

        public static void RemoveItem(ShelfItem item)
        {
            try
            {
                if (File.Exists(item.FilePath))
                {
                    File.Delete(item.FilePath);
                }
                Items.Remove(item);
            }
            catch { }
        }

        public static void ClearAll()
        {
            try
            {
                foreach (var item in Items)
                {
                    if (File.Exists(item.FilePath)) File.Delete(item.FilePath);
                }
                Items.Clear();
            }
            catch { }
        }

        public static void OpenFile(ShelfItem item)
        {
            try
            {
                if (File.Exists(item.FilePath))
                {
                    Process.Start(new ProcessStartInfo(item.FilePath) { UseShellExecute = true });
                }
            }
            catch { }
        }

        public static void ShowInExplorer(ShelfItem item)
        {
            try
            {
                if (File.Exists(item.FilePath))
                {
                    Process.Start("explorer.exe", $"/select,\"{item.FilePath}\"");
                }
            }
            catch { }
        }

        private static string FormatBytes(long bytes)
        {
            string[] suffixes = { "B", "KB", "MB", "GB" };
            int i = 0;
            double d = bytes;
            while (d >= 1024 && i < suffixes.Length - 1)
            {
                d /= 1024;
                i++;
            }
            return $"{d:0.#} {suffixes[i]}";
        }
    }
}
