using System;
using System.Collections.ObjectModel;
using System.Windows;

namespace NotchIsland.Services
{
    public class ClipboardItem
    {
        public string Text { get; set; } = "";
        public DateTime Timestamp { get; set; } = DateTime.Now;
        public bool IsPinned { get; set; }
    }

    public class ClipboardService
    {
        public ObservableCollection<ClipboardItem> Items { get; } = new();
        private string _lastText = "";

        public ClipboardService()
        {
            // Initial seed items matching real NotchPixel snippets
            Items.Add(new ClipboardItem { Text = "https://github.com/abhinavstechdev/NotchIsland", IsPinned = true });
            Items.Add(new ClipboardItem { Text = "cargo build --release", IsPinned = false });
            Items.Add(new ClipboardItem { Text = "NotchIsland v0.4.0 Native Windows App", IsPinned = false });
        }

        public void CheckClipboard()
        {
            try
            {
                if (Clipboard.ContainsText())
                {
                    var text = Clipboard.GetText();
                    if (!string.IsNullOrWhiteSpace(text) && text != _lastText)
                    {
                        _lastText = text;
                        Items.Insert(0, new ClipboardItem { Text = text });
                        if (Items.Count > 30) Items.RemoveAt(Items.Count - 1);
                    }
                }
            }
            catch { }
        }

        public void CopyItem(ClipboardItem item)
        {
            try
            {
                Clipboard.SetText(item.Text);
            }
            catch { }
        }
    }
}
