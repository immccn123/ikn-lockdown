using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.ServiceProcess;

namespace Lockdown.Panel.Services;

public class ServiceManager
{
    private const string ServiceName = "lockdown-service";

    static public void CopyServiceExe()
    {
        string sourcePath = Path.Combine(AppContext.BaseDirectory, "Assets", "lockdown-service.exe");
        string targetPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "lockdown-service.exe");
        File.Copy(sourcePath, targetPath, overwrite: true);
    }

    static public bool ServiceExists() =>
        ServiceController.GetServices().Any(s => s.ServiceName == ServiceName);

    static public void RegisterService()
    {
        try { CopyServiceExe(); } catch { }
        var ServiceExecutablePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "lockdown-service.exe");

        var sc = new ServiceController(ServiceName);
        var serviceInstaller = new System.Configuration.Install.Installer();
        serviceInstaller.Installers.Add(new ServiceInstaller {
            ServiceName = ServiceName,
            StartType = ServiceStartMode.Automatic,
        });
        serviceInstaller.Install(new Dictionary<string, string>());
    }

    static public void UnregisterService() { }

    static public void StartService() { }

    static public bool IsServiceRunning() { throw new NotImplementedException(); }

    public static void StopService() { }
}
