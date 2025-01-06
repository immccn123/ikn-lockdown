using Microsoft.UI.Xaml.Controls;
using ReactiveUI;
using System;
using System.Collections.ObjectModel;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Reactive;
using System.Threading.Tasks;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage.Pickers;

namespace Lockdown.Panel.Pages;

public sealed partial class FileListPage : Page
{
    public FileListViewModel ViewModel { get; }

    public FileListPage()
    {
        this.InitializeComponent();
        ViewModel = new FileListViewModel();
        DataContext = ViewModel;
    }

    private void Grid_DragOver(object sender, Microsoft.UI.Xaml.DragEventArgs e)
    {
        e.AcceptedOperation = DataPackageOperation.Link;
    }

    private async Task AddFileAsync()
    {
        var openPicker = new FileOpenPicker();
        var window = App.Window;
        var hWnd = WinRT.Interop.WindowNative.GetWindowHandle(window);

        WinRT.Interop.InitializeWithWindow.Initialize(openPicker, hWnd);

        openPicker.ViewMode = PickerViewMode.List;
        openPicker.SuggestedStartLocation = PickerLocationId.DocumentsLibrary;
        openPicker.FileTypeFilter.Add("*");

        var files = await openPicker.PickMultipleFilesAsync();
        if (files?.Count > 0) {
            foreach (var file in files) {
                ViewModel.AddFile(file.Path);
            }
            ViewModel.SaveFileItems();
        }
    }
}

public partial class FileListViewModel : ReactiveObject
{
    private bool _isMultipleSelection;
    public bool IsMultipleSelection {
        get => _isMultipleSelection;
        set => this.RaiseAndSetIfChanged(ref _isMultipleSelection, value);
    }

    public ListViewSelectionMode SelectionMode =>
        IsMultipleSelection ? ListViewSelectionMode.Multiple : ListViewSelectionMode.None;

    public ObservableCollection<FileItemViewModel> FileItems { get; } = [];

    public ReactiveCommand<Unit, Unit> DeleteSelectedCommand { get; }

    private readonly string _dataFilePath;

    public FileListViewModel()
    {
        var folderPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData),
            "Lockdown.Service"
        );
        _dataFilePath = Path.Combine(
            folderPath,
            "locked_files.txt"
        );

        if (!Directory.Exists(folderPath)) Directory.CreateDirectory(folderPath);
        if (!File.Exists(_dataFilePath)) File.Create(_dataFilePath).Close();

        DeleteSelectedCommand = ReactiveCommand.Create(DeleteSelectedFiles);

        LoadFileItems();
    }

    private void DeleteSelectedFiles()
    {
        var selectedItems = FileItems.Where(item => item.IsSelected).ToList();
        foreach (var item in selectedItems) {
            FileItems.Remove(item);
        }
        SaveFileItems();
    }

    private void LoadFileItems()
    {
        if (File.Exists(_dataFilePath)) {
            var files = File.ReadAllLines(_dataFilePath);
            foreach (var file in files) {
                FileItems.Add(new FileItemViewModel { FileName = file });
            }
        }
    }

    public void SaveFileItems()
    {
        var files = FileItems.Select(item => item.FileName).ToArray();
        File.WriteAllLines(_dataFilePath, files!);
    }

    public void AddFile([NotNull] string path)
    {
        if (!FileItems.Any(item => item.FileName == path) && !Directory.Exists(path)) {
            FileItems.Add(new FileItemViewModel {
                FileName = path,
                IsSelected = false,
            });
        }
    }

    public async Task DropAsync(object sender, Microsoft.UI.Xaml.DragEventArgs e)
    {
        e.Handled = true;
        var items = await e.DataView.GetStorageItemsAsync();
        foreach (var item in items) {
            var path = item.Path;
            if (path is not null) {
                AddFile(path);
            }
        }
        SaveFileItems();
    }
}

public partial class FileItemViewModel : ReactiveObject
{
    private string? _fileName;
    private bool _isSelected;

    public string? FileName {
        get => _fileName;
        set => this.RaiseAndSetIfChanged(ref _fileName, value);
    }

    public bool IsSelected {
        get => _isSelected;
        set => this.RaiseAndSetIfChanged(ref _isSelected, value);
    }
}
