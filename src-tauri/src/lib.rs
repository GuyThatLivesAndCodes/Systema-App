use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub output: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub start_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartupApp {
    pub name: String,
    pub command: String,
    pub enabled: bool,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_version: String,
    pub computer_name: String,
    pub total_ram_gb: f64,
    pub has_ssd: bool,
}

// Helper function to run PowerShell commands
fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// ============ SYSTEM INFO ============

#[tauri::command]
pub fn get_system_info() -> CommandResult {
    let script = r#"
        $os = Get-CimInstance Win32_OperatingSystem
        $cs = Get-CimInstance Win32_ComputerSystem
        $disk = Get-PhysicalDisk | Where-Object { $_.MediaType -eq 'SSD' -or $_.MediaType -eq 'NVMe' } | Select-Object -First 1

        @{
            os_version = $os.Caption + ' ' + $os.Version
            computer_name = $cs.Name
            total_ram_gb = [math]::Round($cs.TotalPhysicalMemory / 1GB, 2)
            has_ssd = ($null -ne $disk)
        } | ConvertTo-Json
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "System info retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get system info: {}", e),
            output: None,
        },
    }
}

// ============ VIRTUAL MEMORY (RAM SWAP) ============

#[tauri::command]
pub fn get_virtual_memory_info() -> CommandResult {
    let script = r#"
        $pagefile = Get-CimInstance Win32_PageFileUsage | Select-Object -First 1
        $settings = Get-CimInstance Win32_PageFileSetting | Select-Object -First 1

        @{
            current_size_mb = if($pagefile) { $pagefile.AllocatedBaseSize } else { 0 }
            initial_size = if($settings) { $settings.InitialSize } else { 0 }
            maximum_size = if($settings) { $settings.MaximumSize } else { 0 }
            is_auto_managed = (Get-CimInstance Win32_ComputerSystem).AutomaticManagedPagefile
        } | ConvertTo-Json
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Virtual memory info retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get virtual memory info: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn set_virtual_memory(initial_size: u32, max_size: u32) -> CommandResult {
    let script = format!(r#"
        $cs = Get-WmiObject Win32_ComputerSystem -EnableAllPrivileges
        $cs.AutomaticManagedPagefile = $false
        $cs.Put() | Out-Null

        $pf = Get-WmiObject Win32_PageFileSetting
        if ($pf) {{
            $pf.Delete()
        }}

        $newPf = ([WMIClass]"root\cimv2:Win32_PageFileSetting").CreateInstance()
        $newPf.Name = "C:\pagefile.sys"
        $newPf.InitialSize = {}
        $newPf.MaximumSize = {}
        $newPf.Put() | Out-Null

        Write-Output "Virtual memory configured. Restart required for changes to take effect."
    "#, initial_size, max_size);

    match run_powershell(&script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Virtual memory configured successfully. Please restart your computer.".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to set virtual memory: {}", e),
            output: None,
        },
    }
}

// ============ WINDOWS SERVICES ============

#[tauri::command]
pub fn get_services_list() -> CommandResult {
    let script = r#"
        $targetServices = @(
            'lfsvc',           # Geolocation Service
            'PhoneSvc',        # Phone Service
            'Spooler',         # Print Spooler
            'PrintNotify',     # Printer Extensions and Notifications
            'DeviceAssociationBrokerSvc', # Print Device Configuration
            'RemoteRegistry',  # Remote Registry
            'RetailDemo',      # Retail Demo Service
            'seclogon',        # Secondary Logon
            'TimeBrokerSvc',   # Cellular Time
            'WerSvc',          # Windows Error Reporting
            'RasMan',          # Remote Access Connection Manager
            'WpcMonSvc',       # Parental Controls
            'WinRM',           # Windows Remote Management
            'SysMain',         # SysMain/Superfetch
            'DusmSvc'          # Data Usage
        )

        $services = @()
        foreach ($svcName in $targetServices) {
            $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
            if ($svc) {
                $wmiSvc = Get-WmiObject Win32_Service -Filter "Name='$svcName'"
                $services += @{
                    name = $svc.Name
                    display_name = $svc.DisplayName
                    status = $svc.Status.ToString()
                    start_type = if($wmiSvc) { $wmiSvc.StartMode } else { "Unknown" }
                }
            }
        }
        $services | ConvertTo-Json -AsArray
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Services list retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get services: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn set_service_startup(service_name: String, startup_type: String) -> CommandResult {
    let startup_value = match startup_type.as_str() {
        "Disabled" => "Disabled",
        "Manual" => "Manual",
        "Automatic" => "Automatic",
        _ => "Manual",
    };

    let script = format!(r#"
        try {{
            Stop-Service -Name '{}' -Force -ErrorAction SilentlyContinue
            Set-Service -Name '{}' -StartupType {}
            Write-Output "Service '{}' set to {}"
        }} catch {{
            Write-Error $_.Exception.Message
        }}
    "#, service_name, service_name, startup_value, service_name, startup_value);

    match run_powershell(&script) {
        Ok(output) => CommandResult {
            success: true,
            message: format!("Service {} set to {}", service_name, startup_type),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to configure service: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn disable_unnecessary_services() -> CommandResult {
    let script = r#"
        $servicesToDisable = @(
            'lfsvc',           # Geolocation Service
            'PhoneSvc',        # Phone Service
            'RemoteRegistry',  # Remote Registry
            'RetailDemo',      # Retail Demo Service
            'WerSvc',          # Windows Error Reporting
            'DusmSvc'          # Data Usage
        )

        $servicesToManual = @(
            'seclogon'         # Secondary Logon
        )

        $results = @()

        foreach ($svc in $servicesToDisable) {
            try {
                Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
                Set-Service -Name $svc -StartupType Disabled -ErrorAction Stop
                $results += "$svc - Disabled"
            } catch {
                $results += "$svc - Failed: $($_.Exception.Message)"
            }
        }

        foreach ($svc in $servicesToManual) {
            try {
                Set-Service -Name $svc -StartupType Manual -ErrorAction Stop
                $results += "$svc - Set to Manual"
            } catch {
                $results += "$svc - Failed: $($_.Exception.Message)"
            }
        }

        $results -join "`n"
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Unnecessary services configured".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to configure services: {}", e),
            output: None,
        },
    }
}

// ============ STARTUP APPS ============

#[tauri::command]
pub fn get_startup_apps() -> CommandResult {
    let script = r#"
        $startupApps = @()

        # Registry Run keys
        $runKeys = @(
            'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run',
            'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run',
            'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run'
        )

        foreach ($key in $runKeys) {
            if (Test-Path $key) {
                $items = Get-ItemProperty -Path $key -ErrorAction SilentlyContinue
                $items.PSObject.Properties | Where-Object { $_.Name -notlike 'PS*' } | ForEach-Object {
                    $startupApps += @{
                        name = $_.Name
                        command = $_.Value
                        enabled = $true
                        location = $key
                    }
                }
            }
        }

        # Startup folder
        $startupFolder = [Environment]::GetFolderPath('Startup')
        if (Test-Path $startupFolder) {
            Get-ChildItem $startupFolder -File | ForEach-Object {
                $startupApps += @{
                    name = $_.BaseName
                    command = $_.FullName
                    enabled = $true
                    location = "StartupFolder"
                }
            }
        }

        $startupApps | ConvertTo-Json -AsArray
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Startup apps retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get startup apps: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn disable_startup_app(name: String, location: String) -> CommandResult {
    let script = if location == "StartupFolder" {
        format!(r#"
            $startupFolder = [Environment]::GetFolderPath('Startup')
            $file = Get-ChildItem $startupFolder -File | Where-Object {{ $_.BaseName -eq '{}' }}
            if ($file) {{
                Remove-Item $file.FullName -Force
                Write-Output "Removed {} from startup"
            }} else {{
                Write-Output "File not found"
            }}
        "#, name, name)
    } else {
        format!(r#"
            try {{
                Remove-ItemProperty -Path '{}' -Name '{}' -Force -ErrorAction Stop
                Write-Output "Removed {} from startup"
            }} catch {{
                Write-Error $_.Exception.Message
            }}
        "#, location, name, name)
    };

    match run_powershell(&script) {
        Ok(output) => CommandResult {
            success: true,
            message: format!("Disabled {} from startup", name),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to disable startup app: {}", e),
            output: None,
        },
    }
}

// ============ WINDOWS DEFENDER ============

#[tauri::command]
pub fn get_defender_status() -> CommandResult {
    let script = r#"
        try {
            $status = Get-MpComputerStatus -ErrorAction Stop
            @{
                antivirus_enabled = $status.AntivirusEnabled
                real_time_protection = $status.RealTimeProtectionEnabled
                controlled_folder_access = (Get-MpPreference).EnableControlledFolderAccess
            } | ConvertTo-Json
        } catch {
            @{
                antivirus_enabled = $false
                real_time_protection = $false
                controlled_folder_access = 0
            } | ConvertTo-Json
        }
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Defender status retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get Defender status: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn enable_controlled_folder_access() -> CommandResult {
    let script = r#"
        try {
            Set-MpPreference -EnableControlledFolderAccess Enabled
            Write-Output "Controlled Folder Access enabled successfully"
        } catch {
            Write-Error $_.Exception.Message
        }
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Controlled Folder Access enabled".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to enable Controlled Folder Access: {}", e),
            output: None,
        },
    }
}

// ============ POWER SETTINGS ============

#[tauri::command]
pub fn get_power_plan() -> CommandResult {
    let script = r#"
        $activePlan = powercfg /getactivescheme
        $activePlan
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Power plan retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get power plan: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn set_high_performance_power() -> CommandResult {
    let script = r#"
        # Enable High Performance power plan
        $highPerf = powercfg /list | Select-String "High performance"
        if ($highPerf) {
            $guid = ($highPerf -split '\s+')[3]
            powercfg /setactive $guid
            Write-Output "High Performance power plan activated"
        } else {
            # Try Ultimate Performance
            powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61 2>$null
            $ultimate = powercfg /list | Select-String "Ultimate Performance"
            if ($ultimate) {
                $guid = ($ultimate -split '\s+')[3]
                powercfg /setactive $guid
                Write-Output "Ultimate Performance power plan activated"
            } else {
                Write-Output "Could not find High Performance plan"
            }
        }
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "High Performance power plan set".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to set power plan: {}", e),
            output: None,
        },
    }
}

// ============ TELEMETRY SETTINGS ============

#[tauri::command]
pub fn disable_telemetry() -> CommandResult {
    let script = r#"
        $results = @()

        # Disable telemetry in registry
        try {
            $privacyPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Privacy'
            if (-not (Test-Path $privacyPath)) {
                New-Item -Path $privacyPath -Force | Out-Null
            }
            Set-ItemProperty -Path $privacyPath -Name 'TailoredExperiencesWithDiagnosticDataEnabled' -Value 0 -Type DWord -Force
            $results += "Disabled tailored experiences"
        } catch { $results += "Failed to disable tailored experiences" }

        # Disable advertising ID
        try {
            $advertisingPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo'
            if (-not (Test-Path $advertisingPath)) {
                New-Item -Path $advertisingPath -Force | Out-Null
            }
            Set-ItemProperty -Path $advertisingPath -Name 'Enabled' -Value 0 -Type DWord -Force
            $results += "Disabled advertising ID"
        } catch { $results += "Failed to disable advertising ID" }

        # Disable app launch tracking
        try {
            $explorerPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced'
            Set-ItemProperty -Path $explorerPath -Name 'Start_TrackProgs' -Value 0 -Type DWord -Force
            $results += "Disabled app launch tracking"
        } catch { $results += "Failed to disable app launch tracking" }

        # Disable suggested content
        try {
            $contentPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager'
            Set-ItemProperty -Path $contentPath -Name 'SubscribedContent-338393Enabled' -Value 0 -Type DWord -Force
            Set-ItemProperty -Path $contentPath -Name 'SubscribedContent-353694Enabled' -Value 0 -Type DWord -Force
            Set-ItemProperty -Path $contentPath -Name 'SubscribedContent-353696Enabled' -Value 0 -Type DWord -Force
            $results += "Disabled suggested content"
        } catch { $results += "Failed to disable suggested content" }

        $results -join "`n"
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Telemetry settings disabled".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to disable telemetry: {}", e),
            output: None,
        },
    }
}

// ============ DNS SETTINGS ============

#[tauri::command]
pub fn get_dns_settings() -> CommandResult {
    let script = r#"
        $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }
        $dnsInfo = @()

        foreach ($adapter in $adapters) {
            $dns = Get-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue
            $dnsInfo += @{
                adapter_name = $adapter.Name
                interface_index = $adapter.ifIndex
                dns_servers = $dns.ServerAddresses
            }
        }

        $dnsInfo | ConvertTo-Json -AsArray
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "DNS settings retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get DNS settings: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn set_cloudflare_dns() -> CommandResult {
    let script = r#"
        $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }
        $results = @()

        foreach ($adapter in $adapters) {
            try {
                Set-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -ServerAddresses ("1.1.1.1", "1.0.0.1")
                $results += "Set Cloudflare DNS on $($adapter.Name)"
            } catch {
                $results += "Failed on $($adapter.Name): $($_.Exception.Message)"
            }
        }

        $results -join "`n"
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Cloudflare DNS configured".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to set DNS: {}", e),
            output: None,
        },
    }
}

// ============ OPTIONAL FEATURES ============

#[tauri::command]
pub fn get_optional_features() -> CommandResult {
    let script = r#"
        $targetFeatures = @(
            'Internet-Explorer-Optional-amd64',
            'FaxServicesClientPackage',
            'WorkFolders-Client',
            'Printing-Foundation-Features',
            'SMB1Protocol',
            'Windows-Defender-Default-Definitions',
            'MediaPlayback',
            'Microsoft-Hyper-V-All',
            'Microsoft-Windows-Subsystem-Linux',
            'Printing-PrintToPDFServices-Features'
        )

        $features = @()
        $windowsFeatures = Get-WindowsOptionalFeature -Online -ErrorAction SilentlyContinue

        foreach ($feature in $windowsFeatures) {
            if ($targetFeatures -contains $feature.FeatureName -or $feature.State -eq 'Enabled') {
                $features += @{
                    name = $feature.FeatureName
                    display_name = $feature.FeatureName -replace '-', ' '
                    state = $feature.State.ToString()
                }
            }
        }

        $features | ConvertTo-Json -AsArray
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Optional features retrieved".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to get optional features: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn disable_optional_feature(feature_name: String) -> CommandResult {
    let script = format!(r#"
        try {{
            Disable-WindowsOptionalFeature -Online -FeatureName '{}' -NoRestart -ErrorAction Stop
            Write-Output "Feature '{}' disabled. Restart may be required."
        }} catch {{
            Write-Error $_.Exception.Message
        }}
    "#, feature_name, feature_name);

    match run_powershell(&script) {
        Ok(output) => CommandResult {
            success: true,
            message: format!("Feature {} disabled", feature_name),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to disable feature: {}", e),
            output: None,
        },
    }
}

// ============ VISUAL EFFECTS ============

#[tauri::command]
pub fn optimize_visual_effects() -> CommandResult {
    let script = r#"
        $path = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects'
        if (-not (Test-Path $path)) {
            New-Item -Path $path -Force | Out-Null
        }

        # Set to custom (3 = custom, 2 = best appearance, 1 = best performance, 0 = let windows choose)
        Set-ItemProperty -Path $path -Name 'VisualFXSetting' -Value 3 -Type DWord -Force

        $advancedPath = 'HKCU:\Control Panel\Desktop'

        # Disable most animations but keep essential ones
        Set-ItemProperty -Path $advancedPath -Name 'UserPreferencesMask' -Value ([byte[]](0x90,0x12,0x03,0x80,0x10,0x00,0x00,0x00)) -Type Binary -Force

        # Disable window animations
        $dwmPath = 'HKCU:\Software\Microsoft\Windows\DWM'
        Set-ItemProperty -Path $dwmPath -Name 'EnableAeroPeek' -Value 0 -Type DWord -Force -ErrorAction SilentlyContinue

        # Keep font smoothing
        Set-ItemProperty -Path $advancedPath -Name 'FontSmoothing' -Value '2' -Type String -Force

        Write-Output "Visual effects optimized. Some changes may require a restart."
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Visual effects optimized".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to optimize visual effects: {}", e),
            output: None,
        },
    }
}

// ============ QUICK OPTIMIZE ============

#[tauri::command]
pub fn quick_optimize() -> CommandResult {
    let script = r#"
        $results = @()

        # 1. Disable unnecessary services
        $servicesToDisable = @('lfsvc', 'PhoneSvc', 'RemoteRegistry', 'RetailDemo', 'WerSvc', 'DusmSvc')
        foreach ($svc in $servicesToDisable) {
            try {
                Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue
                Set-Service -Name $svc -StartupType Disabled -ErrorAction SilentlyContinue
                $results += "Disabled service: $svc"
            } catch { }
        }

        # 2. Disable telemetry
        try {
            $advertisingPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo'
            if (-not (Test-Path $advertisingPath)) { New-Item -Path $advertisingPath -Force | Out-Null }
            Set-ItemProperty -Path $advertisingPath -Name 'Enabled' -Value 0 -Type DWord -Force
            $results += "Disabled advertising ID"
        } catch { }

        # 3. Set High Performance power plan
        try {
            $highPerf = powercfg /list | Select-String "High performance"
            if ($highPerf) {
                $guid = ($highPerf -split '\s+')[3]
                powercfg /setactive $guid
                $results += "Set High Performance power plan"
            }
        } catch { }

        # 4. Clean temp files
        try {
            Remove-Item "$env:TEMP\*" -Force -Recurse -ErrorAction SilentlyContinue
            Remove-Item "C:\Windows\Temp\*" -Force -Recurse -ErrorAction SilentlyContinue
            $results += "Cleaned temp files"
        } catch { }

        $results -join "`n"
    "#;

    match run_powershell(script) {
        Ok(output) => CommandResult {
            success: true,
            message: "Quick optimization completed".to_string(),
            output: Some(output),
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Quick optimization failed: {}", e),
            output: None,
        },
    }
}

// ============ OPEN EXTERNAL TOOLS ============

#[tauri::command]
pub fn open_system_properties() -> CommandResult {
    match Command::new("cmd")
        .args(["/C", "sysdm.cpl"])
        .spawn()
    {
        Ok(_) => CommandResult {
            success: true,
            message: "System Properties opened".to_string(),
            output: None,
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to open System Properties: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn open_services() -> CommandResult {
    match Command::new("cmd")
        .args(["/C", "services.msc"])
        .spawn()
    {
        Ok(_) => CommandResult {
            success: true,
            message: "Services opened".to_string(),
            output: None,
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to open Services: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn open_task_manager() -> CommandResult {
    match Command::new("cmd")
        .args(["/C", "taskmgr"])
        .spawn()
    {
        Ok(_) => CommandResult {
            success: true,
            message: "Task Manager opened".to_string(),
            output: None,
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to open Task Manager: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn open_optional_features() -> CommandResult {
    match Command::new("cmd")
        .args(["/C", "optionalfeatures"])
        .spawn()
    {
        Ok(_) => CommandResult {
            success: true,
            message: "Optional Features opened".to_string(),
            output: None,
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to open Optional Features: {}", e),
            output: None,
        },
    }
}

#[tauri::command]
pub fn open_windows_security() -> CommandResult {
    match Command::new("cmd")
        .args(["/C", "start", "windowsdefender:"])
        .spawn()
    {
        Ok(_) => CommandResult {
            success: true,
            message: "Windows Security opened".to_string(),
            output: None,
        },
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to open Windows Security: {}", e),
            output: None,
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // System Info
            get_system_info,
            // Virtual Memory
            get_virtual_memory_info,
            set_virtual_memory,
            // Services
            get_services_list,
            set_service_startup,
            disable_unnecessary_services,
            // Startup Apps
            get_startup_apps,
            disable_startup_app,
            // Defender
            get_defender_status,
            enable_controlled_folder_access,
            // Power
            get_power_plan,
            set_high_performance_power,
            // Telemetry
            disable_telemetry,
            // DNS
            get_dns_settings,
            set_cloudflare_dns,
            // Optional Features
            get_optional_features,
            disable_optional_feature,
            // Visual Effects
            optimize_visual_effects,
            // Quick Optimize
            quick_optimize,
            // Open External
            open_system_properties,
            open_services,
            open_task_manager,
            open_optional_features,
            open_windows_security
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
