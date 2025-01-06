using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Lockdown.Panel;

/// <summary>
/// The main window.
/// </summary>
public sealed partial class MainWindow : Window
{

    public MainWindow()
    {
        this.InitializeComponent();
    }

    private void OnNvMainItemInvoked(NavigationView sender, NavigationViewItemInvokedEventArgs args)
    {
        var transitionInfo = args.RecommendedNavigationTransitionInfo;

        if (args.IsSettingsInvoked) {
            contentFrame.Navigate(typeof(Pages.SettingsPage), null, transitionInfo);
            nvMain.Header = "����";
        } else if (args.InvokedItemContainer is not null) {
            string? tag = args.InvokedItemContainer.Tag.ToString();
            switch (tag) {
                case "FileList":
                    contentFrame.Navigate(typeof(Pages.FileListPage), null, transitionInfo);
                    nvMain.Header = "�ļ��б�";
                    break;
            }
        }
    }

    private void OnNvMainLoaded(object sender, RoutedEventArgs e)
    {
        nvMain.SelectedItem = nvMain.MenuItems[0];
        nvMain.Header = "�ļ��б�";
        contentFrame.Navigate(typeof(Pages.FileListPage));
    }
}
