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
        public string FormattedTime => Timestamp.ToString("HH:mm:ss");
    }

    public class ClipboardService
    {
        public ObservableCollection<ClipboardItem> Items { get; } = new();
        private string _lastText = "";

        public ClipboardService()
        {
            Items.Add(new ClipboardItem { Text = "https://github.com/abhinavstechdev/NotchIsland", IsPinned = true });
            Items.Add(new ClipboardItem { Text = "NotchIsland - Native Windows Dynamic Island", IsPinned = true });
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

                        // Check if already in list
                        for (int i = 0; i < Items.Count; i++)
                        {
                            if (Items[i].Text == text)
                            {
                                if (Items[i].IsPinned) return; // Keep pinned where it is
                                Items.RemoveAt(i);
                                break;
                            }
                        }

                        Items.Insert(0, new ClipboardItem { Text = text });
                        int max = SettingsService.Current.ClipboardHistorySize;
                        while (Items.Count > max)
                        {
                            // Remove oldest unpinned
                            int lastUnpinned = -1;
                            for (int i = Items.Count - 1; i >= 0; i--)
                            {
                                if (!Items[i].IsPinned)
                                {
                                    lastUnpinned = i;
                                    break;
                                }
                            }
                            if (lastUnpinned >= 0) Items.RemoveAt(lastUnpinned);
                            else break;
                        }
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
                _lastText = item.Text;
            }
            catch { }
        }

        public void TogglePin(ClipboardItem item)
        {
            item.IsPinned = !item.IsPinned;
        }

        public void DeleteItem(ClipboardItem item)
        {
            Items.Remove(item);
        }

        public void ClearAll()
        {
            for (int i = Items.Count - 1; i >= 0; i--)
            {
                if (!Items[i].IsPinned)
                {
                    Items.RemoveAt(i);
                }
            }
        }
    }
}
