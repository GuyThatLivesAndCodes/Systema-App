// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct CommandResult {
    success: bool,
    message: String,
    output: Option<String>,
}

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

#[tauri::command]
fn get_system_info() -> CommandResult {
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
        Ok(output) => CommandResult { success: true, message: "System info retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn get_virtual_memory_info() -> CommandResult {
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
        Ok(output) => CommandResult { success: true, message: "Virtual memory info retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn set_virtual_memory(initial_size: u32, max_size: u32) -> CommandResult {
    let script = format!(r#"
        $cs = Get-WmiObject Win32_ComputerSystem -EnableAllPrivileges
        $cs.AutomaticManagedPagefile = $false
        $cs.Put() | Out-Null
        $pf = Get-WmiObject Win32_PageFileSetting
        if ($pf) {{ $pf.Delete() }}
        $newPf = ([WMIClass]"root\cimv2:Win32_PageFileSetting").CreateInstance()
        $newPf.Name = "C:\pagefile.sys"
        $newPf.InitialSize = {}
        $newPf.MaximumSize = {}
        $newPf.Put() | Out-Null
        Write-Output "Virtual memory configured. Restart required."
    "#, initial_size, max_size);
    match run_powershell(&script) {
        Ok(output) => CommandResult { success: true, message: "Virtual memory configured. Please restart.".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn get_services_list() -> CommandResult {
    let script = r#"
        $targetServices = @('lfsvc','PhoneSvc','Spooler','PrintNotify','DeviceAssociationBrokerSvc','RemoteRegistry','RetailDemo','seclogon','TimeBrokerSvc','WerSvc','RasMan','WpcMonSvc','WinRM','SysMain','DusmSvc')
        $services = @()
        foreach ($svcName in $targetServices) {
            $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
            if ($svc) {
                $wmiSvc = Get-WmiObject Win32_Service -Filter "Name='$svcName'"
                $services += @{ name = $svc.Name; display_name = $svc.DisplayName; status = $svc.Status.ToString(); start_type = if($wmiSvc) { $wmiSvc.StartMode } else { "Unknown" } }
            }
        }
        $services | ConvertTo-Json -AsArray
    "#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Services list retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn set_service_startup(service_name: String, startup_type: String) -> CommandResult {
    let script = format!(r#"
        try {{ Stop-Service -Name '{}' -Force -ErrorAction SilentlyContinue; Set-Service -Name '{}' -StartupType {}; Write-Output "Done" }} catch {{ Write-Error $_.Exception.Message }}
    "#, service_name, service_name, startup_type);
    match run_powershell(&script) {
        Ok(output) => CommandResult { success: true, message: format!("Service {} set to {}", service_name, startup_type), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn disable_unnecessary_services() -> CommandResult {
    let script = r#"
        $results = @()
        foreach ($svc in @('lfsvc','PhoneSvc','RemoteRegistry','RetailDemo','WerSvc','DusmSvc')) {
            try { Stop-Service -Name $svc -Force -ErrorAction SilentlyContinue; Set-Service -Name $svc -StartupType Disabled -ErrorAction Stop; $results += "$svc - Disabled" } catch { $results += "$svc - Failed" }
        }
        try { Set-Service -Name 'seclogon' -StartupType Manual -ErrorAction Stop; $results += "seclogon - Manual" } catch { }
        $results -join "`n"
    "#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Services configured".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn get_startup_apps() -> CommandResult {
    let script = r#"
        $startupApps = @()
        foreach ($key in @('HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run','HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run','HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run')) {
            if (Test-Path $key) {
                $items = Get-ItemProperty -Path $key -ErrorAction SilentlyContinue
                $items.PSObject.Properties | Where-Object { $_.Name -notlike 'PS*' } | ForEach-Object { $startupApps += @{ name = $_.Name; command = $_.Value; enabled = $true; location = $key } }
            }
        }
        $startupFolder = [Environment]::GetFolderPath('Startup')
        if (Test-Path $startupFolder) { Get-ChildItem $startupFolder -File | ForEach-Object { $startupApps += @{ name = $_.BaseName; command = $_.FullName; enabled = $true; location = "StartupFolder" } } }
        $startupApps | ConvertTo-Json -AsArray
    "#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Startup apps retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn disable_startup_app(name: String, location: String) -> CommandResult {
    let script = if location == "StartupFolder" {
        format!(r#"$f = Get-ChildItem ([Environment]::GetFolderPath('Startup')) -File | Where-Object {{ $_.BaseName -eq '{}' }}; if ($f) {{ Remove-Item $f.FullName -Force }}"#, name)
    } else {
        format!(r#"Remove-ItemProperty -Path '{}' -Name '{}' -Force -ErrorAction Stop"#, location, name)
    };
    match run_powershell(&script) {
        Ok(output) => CommandResult { success: true, message: format!("Disabled {}", name), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn get_defender_status() -> CommandResult {
    let script = r#"try { $s = Get-MpComputerStatus -ErrorAction Stop; @{ antivirus_enabled = $s.AntivirusEnabled; real_time_protection = $s.RealTimeProtectionEnabled; controlled_folder_access = (Get-MpPreference).EnableControlledFolderAccess } | ConvertTo-Json } catch { @{ antivirus_enabled = $false; real_time_protection = $false; controlled_folder_access = 0 } | ConvertTo-Json }"#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Defender status retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn enable_controlled_folder_access() -> CommandResult {
    let script = r#"try { Set-MpPreference -EnableControlledFolderAccess Enabled; Write-Output "Enabled" } catch { Write-Error $_.Exception.Message }"#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Controlled Folder Access enabled".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn get_power_plan() -> CommandResult {
    match run_powershell("powercfg /getactivescheme") {
        Ok(output) => CommandResult { success: true, message: "Power plan retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn set_high_performance_power() -> CommandResult {
    let script = r#"
        $hp = powercfg /list | Select-String "High performance"
        if ($hp) { $guid = ($hp -split '\s+')[3]; powercfg /setactive $guid; Write-Output "High Performance activated" }
        else { powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61 2>$null; $u = powercfg /list | Select-String "Ultimate"; if ($u) { $guid = ($u -split '\s+')[3]; powercfg /setactive $guid; Write-Output "Ultimate Performance activated" } }
    "#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "High Performance set".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn disable_telemetry() -> CommandResult {
    let script = r#"
        $r = @()
        try { $p = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo'; if (-not (Test-Path $p)) { New-Item -Path $p -Force | Out-Null }; Set-ItemProperty -Path $p -Name 'Enabled' -Value 0 -Type DWord -Force; $r += "Disabled ads" } catch { }
        try { Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name 'Start_TrackProgs' -Value 0 -Type DWord -Force; $r += "Disabled tracking" } catch { }
        try { $c = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager'; Set-ItemProperty -Path $c -Name 'SubscribedContent-338393Enabled' -Value 0 -Type DWord -Force; Set-ItemProperty -Path $c -Name 'SubscribedContent-353694Enabled' -Value 0 -Type DWord -Force; $r += "Disabled suggestions" } catch { }
        $r -join "`n"
    "#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Telemetry disabled".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn get_dns_settings() -> CommandResult {
    let script = r#"$adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }; $info = @(); foreach ($a in $adapters) { $dns = Get-DnsClientServerAddress -InterfaceIndex $a.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue; $info += @{ adapter_name = $a.Name; interface_index = $a.ifIndex; dns_servers = $dns.ServerAddresses } }; $info | ConvertTo-Json -AsArray"#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "DNS settings retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn set_cloudflare_dns() -> CommandResult {
    let script = r#"$r = @(); foreach ($a in (Get-NetAdapter | Where-Object { $_.Status -eq 'Up' })) { try { Set-DnsClientServerAddress -InterfaceIndex $a.ifIndex -ServerAddresses ("1.1.1.1","1.0.0.1"); $r += "Set DNS on $($a.Name)" } catch { $r += "Failed on $($a.Name)" } }; $r -join "`n""#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Cloudflare DNS configured".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn get_optional_features() -> CommandResult {
    let script = r#"$f = @(); Get-WindowsOptionalFeature -Online -ErrorAction SilentlyContinue | ForEach-Object { if ($_.State -eq 'Enabled') { $f += @{ name = $_.FeatureName; display_name = $_.FeatureName -replace '-',' '; state = $_.State.ToString() } } }; $f | ConvertTo-Json -AsArray"#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Features retrieved".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn disable_optional_feature(feature_name: String) -> CommandResult {
    let script = format!(r#"try {{ Disable-WindowsOptionalFeature -Online -FeatureName '{}' -NoRestart -ErrorAction Stop; Write-Output "Disabled" }} catch {{ Write-Error $_.Exception.Message }}"#, feature_name);
    match run_powershell(&script) {
        Ok(output) => CommandResult { success: true, message: format!("Feature {} disabled", feature_name), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn optimize_visual_effects() -> CommandResult {
    let script = r#"
        $p = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects'; if (-not (Test-Path $p)) { New-Item -Path $p -Force | Out-Null }
        Set-ItemProperty -Path $p -Name 'VisualFXSetting' -Value 3 -Type DWord -Force
        Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'UserPreferencesMask' -Value ([byte[]](0x90,0x12,0x03,0x80,0x10,0x00,0x00,0x00)) -Type Binary -Force
        Set-ItemProperty -Path 'HKCU:\Control Panel\Desktop' -Name 'FontSmoothing' -Value '2' -Type String -Force
        Write-Output "Visual effects optimized"
    "#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Visual effects optimized".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn quick_optimize() -> CommandResult {
    let script = r#"
        $r = @()
        foreach ($s in @('lfsvc','PhoneSvc','RemoteRegistry','RetailDemo','WerSvc','DusmSvc')) { try { Stop-Service -Name $s -Force -ErrorAction SilentlyContinue; Set-Service -Name $s -StartupType Disabled -ErrorAction SilentlyContinue; $r += "Disabled $s" } catch { } }
        try { $p = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo'; if (-not (Test-Path $p)) { New-Item -Path $p -Force | Out-Null }; Set-ItemProperty -Path $p -Name 'Enabled' -Value 0 -Type DWord -Force; $r += "Disabled ads" } catch { }
        try { $hp = powercfg /list | Select-String "High performance"; if ($hp) { $guid = ($hp -split '\s+')[3]; powercfg /setactive $guid; $r += "Set High Performance" } } catch { }
        try { Remove-Item "$env:TEMP\*" -Force -Recurse -ErrorAction SilentlyContinue; $r += "Cleaned temp" } catch { }
        $r -join "`n"
    "#;
    match run_powershell(script) {
        Ok(output) => CommandResult { success: true, message: "Quick optimization completed".to_string(), output: Some(output) },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn open_system_properties() -> CommandResult {
    match Command::new("cmd").args(["/C", "sysdm.cpl"]).spawn() {
        Ok(_) => CommandResult { success: true, message: "Opened".to_string(), output: None },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn open_services() -> CommandResult {
    match Command::new("cmd").args(["/C", "services.msc"]).spawn() {
        Ok(_) => CommandResult { success: true, message: "Opened".to_string(), output: None },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn open_task_manager() -> CommandResult {
    match Command::new("cmd").args(["/C", "taskmgr"]).spawn() {
        Ok(_) => CommandResult { success: true, message: "Opened".to_string(), output: None },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn open_optional_features() -> CommandResult {
    match Command::new("cmd").args(["/C", "optionalfeatures"]).spawn() {
        Ok(_) => CommandResult { success: true, message: "Opened".to_string(), output: None },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

#[tauri::command]
fn open_windows_security() -> CommandResult {
    match Command::new("cmd").args(["/C", "start", "windowsdefender:"]).spawn() {
        Ok(_) => CommandResult { success: true, message: "Opened".to_string(), output: None },
        Err(e) => CommandResult { success: false, message: format!("Failed: {}", e), output: None },
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            get_virtual_memory_info,
            set_virtual_memory,
            get_services_list,
            set_service_startup,
            disable_unnecessary_services,
            get_startup_apps,
            disable_startup_app,
            get_defender_status,
            enable_controlled_folder_access,
            get_power_plan,
            set_high_performance_power,
            disable_telemetry,
            get_dns_settings,
            set_cloudflare_dns,
            get_optional_features,
            disable_optional_feature,
            optimize_visual_effects,
            quick_optimize,
            open_system_properties,
            open_services,
            open_task_manager,
            open_optional_features,
            open_windows_security
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
