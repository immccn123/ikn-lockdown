using System.Runtime.InteropServices;
using Windows.Win32;
using Windows.Win32.Foundation;
using Windows.Win32.Storage.FileSystem;

namespace Lockdown.Service;

public class WindowsWorker(ILogger<WindowsWorker> logger) : BackgroundService
{
    private readonly HashSet<(string filePath, SafeHandle handle)> fileHandles = [];
    private readonly string DataFilePath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData),
        "Lockdown.Service",
        "locked_files.txt"
    );


    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        LoadAndLockFiles();

        try {
            while (!stoppingToken.IsCancellationRequested) {
                await Task.Delay(1000, stoppingToken);
            }
        } catch (OperationCanceledException) {
            UnlockAllFiles();
        }
    }

    /// <summary>
    /// 从数据文件加载文件列表并锁定文件
    /// </summary>
    private void LoadAndLockFiles()
    {
        if (File.Exists(DataFilePath)) {
            var files = File.ReadAllLines(DataFilePath);
            foreach (var _file in files) {
                var file = _file.Trim();
                if (!string.IsNullOrWhiteSpace(file) && File.Exists(file)) {
                    LockFile(file);
                }
            }
        }
    }

    /// <summary>
    /// 锁定文件
    /// </summary>
    private void LockFile(string filePath)
    {
        // 使用打开文件并设置独占访问权限
        var handle = PInvoke.CreateFile(
            filePath,
            (uint)(GENERIC_ACCESS_RIGHTS.GENERIC_READ | GENERIC_ACCESS_RIGHTS.GENERIC_WRITE),
            FILE_SHARE_MODE.FILE_SHARE_READ,
            null,
            FILE_CREATION_DISPOSITION.OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES.FILE_ATTRIBUTE_NORMAL,
            null
        );


        if (handle.IsInvalid) {
            logger.LogError("文件锁定失败: {file}", filePath);
        } else {
            fileHandles.Add((filePath, handle));
            logger.LogInformation("文件已锁定: {file}", filePath);
        }
    }

    /// <summary>
    /// 解锁所有文件
    /// </summary>
    private void UnlockAllFiles()
    {
        foreach (var (filePath, handle) in fileHandles) {
            handle.Dispose();
            logger.LogInformation("文件已解锁: {file}", filePath);
        }
        fileHandles.Clear();
    }
}