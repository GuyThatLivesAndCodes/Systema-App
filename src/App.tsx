import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";

interface CommandResult {
  success: boolean;
  message: string;
  output?: string;
}

interface Toast {
  id: number;
  message: string;
  type: "success" | "error" | "warning";
}

type Page = "dashboard" | "memory" | "services" | "startup" | "power" | "privacy" | "network" | "features" | "visual" | "tools" | "guides";

const serviceDescriptions: Record<string, string> = {
  lfsvc: "Geolocation Service - Location tracking for apps",
  PhoneSvc: "Phone Service - Legacy phone features",
  Spooler: "Print Spooler - Manages print jobs",
  PrintNotify: "Printer Extensions and Notifications",
  DeviceAssociationBrokerSvc: "Print Device Configuration Service",
  RemoteRegistry: "Remote Registry - Network registry access",
  RetailDemo: "Retail Demo Service - Store display mode",
  seclogon: "Secondary Logon - Run as different user",
  TimeBrokerSvc: "Cellular Time - Mobile time sync",
  WerSvc: "Windows Error Reporting",
  RasMan: "Remote Access Connection Manager",
  WpcMonSvc: "Parental Controls - Family safety monitoring",
  WinRM: "Windows Remote Management",
  SysMain: "SysMain/Superfetch - Preloading optimization",
  DusmSvc: "Data Usage - Internet usage monitoring",
};

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("dashboard");
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [loading, setLoading] = useState(false);
  const [services, setServices] = useState<any[]>([]);
  const [startupApps, setStartupApps] = useState<any[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [modalContent] = useState<{ title: string; content: React.ReactNode } | null>(null);
  const [vmInitialSize, setVmInitialSize] = useState("32768");
  const [vmMaxSize, setVmMaxSize] = useState("32768");

  const addToast = (message: string, type: "success" | "error" | "warning") => {
    const id = Date.now();
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4000);
  };

  const runCommand = async (command: string, args?: any) => {
    setLoading(true);
    try {
      const result = await invoke<CommandResult>(command, args || {});
      if (result.success) {
        addToast(result.message, "success");
      } else {
        addToast(result.message, "error");
      }
      return result;
    } catch (error) {
      addToast(`Error: ${error}`, "error");
      return null;
    } finally {
      setLoading(false);
    }
  };

  const loadServices = async () => {
    try {
      const result = await invoke<CommandResult>("get_services_list");
      if (result.success && result.output) {
        const parsed = JSON.parse(result.output);
        setServices(Array.isArray(parsed) ? parsed : []);
      }
    } catch (error) {
      console.error("Failed to load services:", error);
    }
  };

  const loadStartupApps = async () => {
    try {
      const result = await invoke<CommandResult>("get_startup_apps");
      if (result.success && result.output) {
        const parsed = JSON.parse(result.output);
        setStartupApps(Array.isArray(parsed) ? parsed : []);
      }
    } catch (error) {
      console.error("Failed to load startup apps:", error);
    }
  };

  useEffect(() => {
    if (currentPage === "services") loadServices();
    if (currentPage === "startup") loadStartupApps();
  }, [currentPage]);

  const handleQuickOptimize = async () => {
    setLoading(true);
    try {
      await runCommand("quick_optimize");
    } finally {
      setLoading(false);
    }
  };

  const openExternalUrl = async (url: string) => {
    try {
      await openUrl(url);
    } catch (error) {
      addToast(`Failed to open URL: ${error}`, "error");
    }
  };

  const renderDashboard = () => (
    <>
      <div className="page-header">
        <h2>Dashboard</h2>
        <p>Welcome to Systema - Your Windows optimization toolkit</p>
      </div>

      <div className="info-box info">
        <span className="info-box-icon">&#9432;</span>
        <div className="info-box-content">
          <strong>Getting Started</strong>
          Use the sidebar to navigate between different optimization categories.
          The Quick Optimize button applies safe, recommended optimizations automatically.
        </div>
      </div>

      <div className="card-grid">
        <div className="card" onClick={() => setCurrentPage("memory")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon blue">&#128190;</div>
              <h3>Virtual Memory</h3>
            </div>
          </div>
          <p className="card-description">
            Configure RAM swap settings to use your SSD as backup memory, preventing crashes during heavy usage.
          </p>
          <button className="btn btn-secondary">Configure</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("services")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon orange">&#9881;</div>
              <h3>Windows Services</h3>
            </div>
          </div>
          <p className="card-description">
            Disable unnecessary Windows services to free up system resources and reduce background activity.
          </p>
          <button className="btn btn-secondary">Manage</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("startup")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon green">&#128640;</div>
              <h3>Startup Apps</h3>
            </div>
          </div>
          <p className="card-description">
            Control which applications launch at startup to improve boot times and reduce resource usage.
          </p>
          <button className="btn btn-secondary">Manage</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("power")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon blue">&#9889;</div>
              <h3>Power Settings</h3>
            </div>
          </div>
          <p className="card-description">
            Switch to high-performance power mode for maximum system performance when plugged in.
          </p>
          <button className="btn btn-secondary">Configure</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("privacy")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon red">&#128274;</div>
              <h3>Privacy & Telemetry</h3>
            </div>
          </div>
          <p className="card-description">
            Disable Windows telemetry, advertising ID, and other privacy-invasive settings.
          </p>
          <button className="btn btn-secondary">Configure</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("network")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon green">&#127760;</div>
              <h3>DNS Settings</h3>
            </div>
          </div>
          <p className="card-description">
            Configure Cloudflare DNS for faster and more private internet browsing.
          </p>
          <button className="btn btn-secondary">Configure</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("features")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon orange">&#128230;</div>
              <h3>Optional Features</h3>
            </div>
          </div>
          <p className="card-description">
            Remove unused Windows features like Internet Explorer, Fax, and legacy components.
          </p>
          <button className="btn btn-secondary">Manage</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("visual")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon blue">&#127912;</div>
              <h3>Visual Effects</h3>
            </div>
          </div>
          <p className="card-description">
            Reduce Windows animations for better performance while keeping essential visual elements.
          </p>
          <button className="btn btn-secondary">Optimize</button>
        </div>

        <div className="card" onClick={() => setCurrentPage("guides")}>
          <div className="card-header">
            <div className="card-title">
              <div className="card-icon green">&#128214;</div>
              <h3>External Tools Guide</h3>
            </div>
          </div>
          <p className="card-description">
            Step-by-step guides for Process Lasso, Revo Uninstaller, and other recommended tools.
          </p>
          <button className="btn btn-secondary">View Guides</button>
        </div>
      </div>
    </>
  );

  const renderMemory = () => (
    <>
      <div className="page-header">
        <h2>Virtual Memory (RAM Swap)</h2>
        <p>Configure your SSD as backup memory to prevent crashes during heavy usage</p>
      </div>

      <div className="info-box warning">
        <span className="info-box-icon">&#9888;</span>
        <div className="info-box-content">
          <strong>Important</strong>
          Only use this if you have an SSD, NOT a hard drive! This feature uses your SSD as virtual memory
          which helps prevent crashes when RAM is full.
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <div className="card-icon blue">&#128190;</div>
            <h3>Configure Virtual Memory</h3>
          </div>
        </div>
        <p className="card-description">
          Set the initial and maximum size for your page file. Recommended: 32768 MB if you have enough storage,
          otherwise use 16000 MB.
        </p>

        <div className="form-group">
          <label className="form-label">Initial Size (MB)</label>
          <input
            type="number"
            className="form-input"
            value={vmInitialSize}
            onChange={(e) => setVmInitialSize(e.target.value)}
            placeholder="32768"
          />
          <p className="form-hint">Recommended: 32768 for 32GB, 16000 for less storage</p>
        </div>

        <div className="form-group">
          <label className="form-label">Maximum Size (MB)</label>
          <input
            type="number"
            className="form-input"
            value={vmMaxSize}
            onChange={(e) => setVmMaxSize(e.target.value)}
            placeholder="32768"
          />
          <p className="form-hint">Set same as initial size for best performance</p>
        </div>

        <div className="btn-group">
          <button
            className="btn btn-primary"
            onClick={() => runCommand("set_virtual_memory", {
              initialSize: parseInt(vmInitialSize),
              maxSize: parseInt(vmMaxSize)
            })}
            disabled={loading}
          >
            Apply Settings
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => runCommand("open_system_properties")}
          >
            Open System Properties
          </button>
        </div>
      </div>

      <div className="info-box info">
        <span className="info-box-icon">&#9432;</span>
        <div className="info-box-content">
          <strong>Manual Method</strong>
          Press Win+R, type "sysdm.cpl", go to Advanced tab, click Settings under Performance,
          Advanced tab, then Change under Virtual Memory.
        </div>
      </div>
    </>
  );

  const renderServices = () => (
    <>
      <div className="page-header">
        <h2>Windows Services</h2>
        <p>Disable or set to manual unnecessary Windows services</p>
      </div>

      <div className="info-box warning">
        <span className="info-box-icon">&#9888;</span>
        <div className="info-box-content">
          <strong>Caution</strong>
          Read the description of each service before disabling. Some services may be needed for specific features.
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <h3>Quick Actions</h3>
          </div>
        </div>
        <div className="btn-group">
          <button
            className="btn btn-primary"
            onClick={async () => {
              await runCommand("disable_unnecessary_services");
              loadServices();
            }}
            disabled={loading}
          >
            Disable Recommended Services
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => runCommand("open_services")}
          >
            Open Services Manager
          </button>
          <button
            className="btn btn-secondary"
            onClick={loadServices}
          >
            Refresh List
          </button>
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <h3>Services List</h3>
          </div>
        </div>
        <div className="service-list">
          {services.length === 0 ? (
            <p className="card-description">Loading services... Click "Refresh List" if not loaded.</p>
          ) : (
            services.map((service: any) => (
              <div key={service.name} className="service-item">
                <div className="service-info">
                  <div className="service-name">{service.display_name}</div>
                  <div className="service-desc">
                    {serviceDescriptions[service.name] || service.name}
                  </div>
                </div>
                <div className="service-actions">
                  <span className={`status-badge ${service.start_type === "Disabled" ? "success" : service.start_type === "Manual" ? "warning" : "info"}`}>
                    {service.start_type}
                  </span>
                  <select
                    onChange={async (e) => {
                      await runCommand("set_service_startup", {
                        serviceName: service.name,
                        startupType: e.target.value,
                      });
                      loadServices();
                    }}
                    defaultValue=""
                  >
                    <option value="" disabled>Change</option>
                    <option value="Disabled">Disabled</option>
                    <option value="Manual">Manual</option>
                    <option value="Automatic">Automatic</option>
                  </select>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </>
  );

  const renderStartup = () => (
    <>
      <div className="page-header">
        <h2>Startup Applications</h2>
        <p>Manage applications that launch when Windows starts</p>
      </div>

      <div className="info-box info">
        <span className="info-box-icon">&#9432;</span>
        <div className="info-box-content">
          <strong>Tip</strong>
          Disabling unnecessary startup apps can significantly improve boot times and reduce background resource usage.
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <h3>Quick Actions</h3>
          </div>
        </div>
        <div className="btn-group">
          <button
            className="btn btn-secondary"
            onClick={() => runCommand("open_task_manager")}
          >
            Open Task Manager
          </button>
          <button
            className="btn btn-secondary"
            onClick={loadStartupApps}
          >
            Refresh List
          </button>
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <h3>Startup Apps</h3>
          </div>
        </div>
        <div className="service-list">
          {startupApps.length === 0 ? (
            <p className="card-description">Loading startup apps... Click "Refresh List" if not loaded.</p>
          ) : (
            startupApps.map((app: any, index: number) => (
              <div key={index} className="service-item">
                <div className="service-info">
                  <div className="service-name">{app.name}</div>
                  <div className="service-desc">{app.command?.substring(0, 60)}...</div>
                </div>
                <div className="service-actions">
                  <span className="status-badge success">Enabled</span>
                  <button
                    className="btn btn-danger"
                    onClick={async () => {
                      await runCommand("disable_startup_app", {
                        name: app.name,
                        location: app.location,
                      });
                      loadStartupApps();
                    }}
                  >
                    Disable
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </>
  );

  const renderPower = () => (
    <>
      <div className="page-header">
        <h2>Power Settings</h2>
        <p>Optimize power settings for maximum performance</p>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <div className="card-icon blue">&#9889;</div>
            <h3>High Performance Mode</h3>
          </div>
        </div>
        <p className="card-description">
          Switch to the High Performance power plan to get maximum performance from your PC.
          This may increase power consumption but provides the best performance when plugged in.
        </p>
        <div className="btn-group">
          <button
            className="btn btn-primary"
            onClick={() => runCommand("set_high_performance_power")}
            disabled={loading}
          >
            Enable High Performance
          </button>
        </div>
      </div>

      <div className="info-box info">
        <span className="info-box-icon">&#9432;</span>
        <div className="info-box-content">
          <strong>Manual Method</strong>
          Press Win+I to open Settings, go to System, then Power & battery,
          and change "Power mode" when plugged in to "Best performance".
        </div>
      </div>
    </>
  );

  const renderPrivacy = () => (
    <>
      <div className="page-header">
        <h2>Privacy & Telemetry</h2>
        <p>Disable Windows tracking and telemetry settings</p>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <div className="card-icon red">&#128274;</div>
            <h3>Disable Telemetry</h3>
          </div>
        </div>
        <p className="card-description">
          Disable Windows advertising ID, app launch tracking, tailored experiences,
          and suggested content. This improves privacy and may slightly improve performance.
        </p>
        <div className="btn-group">
          <button
            className="btn btn-primary"
            onClick={() => runCommand("disable_telemetry")}
            disabled={loading}
          >
            Disable All Telemetry
          </button>
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <div className="card-icon green">&#128737;</div>
            <h3>Windows Defender</h3>
          </div>
        </div>
        <p className="card-description">
          Enable Controlled Folder Access to protect your important files from ransomware.
          This is recommended if you don't have another antivirus solution.
        </p>
        <div className="btn-group">
          <button
            className="btn btn-success"
            onClick={() => runCommand("enable_controlled_folder_access")}
            disabled={loading}
          >
            Enable Controlled Folder Access
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => runCommand("open_windows_security")}
          >
            Open Windows Security
          </button>
        </div>
      </div>

      <div className="info-box info">
        <span className="info-box-icon">&#9432;</span>
        <div className="info-box-content">
          <strong>Manual Method</strong>
          Press Win+I, go to Privacy & security, then General, and disable all options.
        </div>
      </div>
    </>
  );

  const renderNetwork = () => (
    <>
      <div className="page-header">
        <h2>DNS Settings</h2>
        <p>Configure DNS for faster and more private browsing</p>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <div className="card-icon green">&#127760;</div>
            <h3>Cloudflare DNS</h3>
          </div>
        </div>
        <p className="card-description">
          Switch to Cloudflare DNS (1.1.1.1) for faster DNS resolution and improved privacy.
          Cloudflare DNS is one of the fastest and most privacy-focused DNS providers.
        </p>
        <div className="info-box info">
          <span className="info-box-icon">&#9432;</span>
          <div className="info-box-content">
            Primary DNS: <strong>1.1.1.1</strong><br />
            Secondary DNS: <strong>1.0.0.1</strong>
          </div>
        </div>
        <div className="btn-group">
          <button
            className="btn btn-primary"
            onClick={() => runCommand("set_cloudflare_dns")}
            disabled={loading}
          >
            Apply Cloudflare DNS
          </button>
        </div>
      </div>

      <div className="info-box warning">
        <span className="info-box-icon">&#9888;</span>
        <div className="info-box-content">
          <strong>Manual Method</strong>
          Open Settings, go to Network & Internet, select your connection (Wi-Fi or Ethernet),
          click Hardware properties, change DNS to Manual, enable IPv4, and enter the DNS addresses above.
          Enable DNS over HTTPS for additional security.
        </div>
      </div>
    </>
  );

  const renderFeatures = () => (
    <>
      <div className="page-header">
        <h2>Optional Features</h2>
        <p>Remove unused Windows features to reduce attack surface and improve performance</p>
      </div>

      <div className="info-box warning">
        <span className="info-box-icon">&#9888;</span>
        <div className="info-box-content">
          <strong>Note</strong>
          Windows will offer to reinstall features if needed in the future.
          It's safe to remove features you don't use.
        </div>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <h3>Commonly Unused Features</h3>
          </div>
        </div>
        <div className="checkbox-list">
          <div className="checkbox-item">
            <div className="checkbox-label">
              <span>Internet Explorer 11</span>
              <small>Legacy browser still lurking in the background</small>
            </div>
          </div>
          <div className="checkbox-item">
            <div className="checkbox-label">
              <span>Windows Fax and Scan</span>
              <small>For faxing from your PC (rarely used)</small>
            </div>
          </div>
          <div className="checkbox-item">
            <div className="checkbox-label">
              <span>Work Folders Client</span>
              <small>Only used in enterprise setups</small>
            </div>
          </div>
          <div className="checkbox-item">
            <div className="checkbox-label">
              <span>SMB 1.0/CIFS</span>
              <small>Ancient, insecure - only keep for XP machines</small>
            </div>
          </div>
          <div className="checkbox-item">
            <div className="checkbox-label">
              <span>Hyper-V</span>
              <small>Only if you don't do virtual machine work</small>
            </div>
          </div>
          <div className="checkbox-item">
            <div className="checkbox-label">
              <span>Windows Subsystem for Linux</span>
              <small>Only if you don't do Linux development</small>
            </div>
          </div>
        </div>
        <div className="btn-group" style={{ marginTop: "16px" }}>
          <button
            className="btn btn-secondary"
            onClick={() => runCommand("open_optional_features")}
          >
            Open Optional Features
          </button>
        </div>
      </div>
    </>
  );

  const renderVisual = () => (
    <>
      <div className="page-header">
        <h2>Visual Effects</h2>
        <p>Reduce animations for better performance</p>
      </div>

      <div className="card">
        <div className="card-header">
          <div className="card-title">
            <div className="card-icon blue">&#127912;</div>
            <h3>Optimize Visual Effects</h3>
          </div>
        </div>
        <p className="card-description">
          Disable most Windows animations while keeping essential visual elements like smooth font rendering
          and window contents while dragging. This can noticeably improve performance on older hardware.
        </p>
        <div className="info-box info">
          <span className="info-box-icon">&#9432;</span>
          <div className="info-box-content">
            <strong>Kept Effects:</strong><br />
            - Animate controls and elements inside windows<br />
            - Show window contents while dragging<br />
            - Smooth edges of screen fonts
          </div>
        </div>
        <div className="btn-group">
          <button
            className="btn btn-primary"
            onClick={() => runCommand("optimize_visual_effects")}
            disabled={loading}
          >
            Optimize Visual Effects
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => runCommand("open_system_properties")}
          >
            Open System Properties
          </button>
        </div>
      </div>

      <div className="info-box warning">
        <span className="info-box-icon">&#9888;</span>
        <div className="info-box-content">
          <strong>Manual Method</strong>
          Press Win+R, type "sysdm.cpl", go to Advanced tab, click Settings under Performance,
          select Custom, and manually choose which effects to keep.
        </div>
      </div>
    </>
  );

  const renderGuides = () => (
    <>
      <div className="page-header">
        <h2>External Tools Guide</h2>
        <p>Step-by-step guides for recommended third-party optimization tools</p>
      </div>

      <div className="guide-section">
        <h3>
          <span style={{ fontSize: "24px" }}>&#128736;</span>
          Process Lasso
        </h3>
        <p className="card-description" style={{ marginBottom: "16px" }}>
          Process Lasso automatically tweaks CPU priorities and core usage so one greedy app doesn't choke your system.
          The ProBalance feature is especially useful for maintaining responsiveness.
        </p>
        <button
          className="btn btn-primary"
          onClick={() => openExternalUrl("https://bitsum.com/download-process-lasso/")}
          style={{ marginBottom: "20px" }}
        >
          Download Process Lasso
        </button>
        <ul className="guide-steps">
          <li className="guide-step">
            <span className="step-number">1</span>
            <div className="step-content">
              <p>Download and install Process Lasso from the official website</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">2</span>
            <div className="step-content">
              <p>Open Process Lasso, click <strong>Main</strong> in the top left</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">3</span>
            <div className="step-content">
              <p>Enable <strong>SmartTrim</strong> - this optimizes RAM smartly, similar to macOS</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">4</span>
            <div className="step-content">
              <p>Go to <strong>Options &gt; CPU &gt; ProBalance Advanced Options</strong></p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">5</span>
            <div className="step-content">
              <p>Configure ProBalance to be more strict for better performance optimization.
              Lower the thresholds so it starts optimizing sooner.</p>
            </div>
          </li>
        </ul>
        <div className="info-box warning" style={{ marginTop: "16px" }}>
          <span className="info-box-icon">&#9888;</span>
          <div className="info-box-content">
            Don't change other Process Lasso settings unless you know what you're doing -
            incorrect settings could hurt performance.
          </div>
        </div>
      </div>

      <div className="guide-section">
        <h3>
          <span style={{ fontSize: "24px" }}>&#128465;</span>
          Revo Uninstaller
        </h3>
        <p className="card-description" style={{ marginBottom: "16px" }}>
          Revo Uninstaller fully removes applications including leftover files and registry entries.
          It can also remove built-in Windows apps like Bing and Edge.
        </p>
        <button
          className="btn btn-primary"
          onClick={() => openExternalUrl("https://www.revouninstaller.com/revo-uninstaller-free-download/")}
          style={{ marginBottom: "20px" }}
        >
          Download Revo Uninstaller
        </button>
        <ul className="guide-steps">
          <li className="guide-step">
            <span className="step-number">1</span>
            <div className="step-content">
              <p>Download and install Revo Uninstaller from the official website</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">2</span>
            <div className="step-content">
              <p>Launch Revo Uninstaller and go to the <strong>Windows Apps</strong> tab</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">3</span>
            <div className="step-content">
              <p>Double-click on the app you want to remove (e.g., Microsoft Edge)</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">4</span>
            <div className="step-content">
              <p>Click <strong>Continue</strong>, wait for the scan, then click <strong>Scan</strong></p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">5</span>
            <div className="step-content">
              <p>Select all leftover items and click <strong>Delete</strong>, then <strong>Finish</strong></p>
            </div>
          </li>
        </ul>
        <div className="info-box info" style={{ marginTop: "16px" }}>
          <span className="info-box-icon">&#9432;</span>
          <div className="info-box-content">
            Before removing Microsoft Edge, make sure you have another browser installed
            (Firefox or Chrome recommended).
          </div>
        </div>
      </div>

      <div className="guide-section">
        <h3>
          <span style={{ fontSize: "24px" }}>&#127919;</span>
          Removing Microsoft Edge
        </h3>
        <p className="card-description" style={{ marginBottom: "16px" }}>
          Microsoft Edge collects significant data. Use Revo Uninstaller to fully remove it.
        </p>
        <ul className="guide-steps">
          <li className="guide-step">
            <span className="step-number">1</span>
            <div className="step-content">
              <p>First, download and install <strong>Firefox</strong> or <strong>Chrome</strong></p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">2</span>
            <div className="step-content">
              <p>Open Revo Uninstaller and go to the <strong>Windows Apps</strong> tab</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">3</span>
            <div className="step-content">
              <p>Find and double-click <strong>Microsoft Edge</strong></p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">4</span>
            <div className="step-content">
              <p>Follow the uninstall process and scan for leftovers</p>
            </div>
          </li>
          <li className="guide-step">
            <span className="step-number">5</span>
            <div className="step-content">
              <p>Select all leftovers and delete them</p>
            </div>
          </li>
        </ul>
      </div>
    </>
  );

  const renderPage = () => {
    switch (currentPage) {
      case "dashboard": return renderDashboard();
      case "memory": return renderMemory();
      case "services": return renderServices();
      case "startup": return renderStartup();
      case "power": return renderPower();
      case "privacy": return renderPrivacy();
      case "network": return renderNetwork();
      case "features": return renderFeatures();
      case "visual": return renderVisual();
      case "guides": return renderGuides();
      default: return renderDashboard();
    }
  };

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="logo">
            <div className="logo-icon">S</div>
            <h1>Systema</h1>
          </div>
        </div>

        <nav className="nav">
          <div className="nav-section">
            <div className="nav-section-title">Overview</div>
            <div
              className={`nav-item ${currentPage === "dashboard" ? "active" : ""}`}
              onClick={() => setCurrentPage("dashboard")}
            >
              <span className="nav-icon">&#127968;</span>
              Dashboard
            </div>
          </div>

          <div className="nav-section">
            <div className="nav-section-title">System</div>
            <div
              className={`nav-item ${currentPage === "memory" ? "active" : ""}`}
              onClick={() => setCurrentPage("memory")}
            >
              <span className="nav-icon">&#128190;</span>
              Virtual Memory
            </div>
            <div
              className={`nav-item ${currentPage === "services" ? "active" : ""}`}
              onClick={() => setCurrentPage("services")}
            >
              <span className="nav-icon">&#9881;</span>
              Services
            </div>
            <div
              className={`nav-item ${currentPage === "startup" ? "active" : ""}`}
              onClick={() => setCurrentPage("startup")}
            >
              <span className="nav-icon">&#128640;</span>
              Startup Apps
            </div>
            <div
              className={`nav-item ${currentPage === "power" ? "active" : ""}`}
              onClick={() => setCurrentPage("power")}
            >
              <span className="nav-icon">&#9889;</span>
              Power
            </div>
          </div>

          <div className="nav-section">
            <div className="nav-section-title">Privacy</div>
            <div
              className={`nav-item ${currentPage === "privacy" ? "active" : ""}`}
              onClick={() => setCurrentPage("privacy")}
            >
              <span className="nav-icon">&#128274;</span>
              Telemetry
            </div>
            <div
              className={`nav-item ${currentPage === "network" ? "active" : ""}`}
              onClick={() => setCurrentPage("network")}
            >
              <span className="nav-icon">&#127760;</span>
              DNS Settings
            </div>
          </div>

          <div className="nav-section">
            <div className="nav-section-title">Optimization</div>
            <div
              className={`nav-item ${currentPage === "features" ? "active" : ""}`}
              onClick={() => setCurrentPage("features")}
            >
              <span className="nav-icon">&#128230;</span>
              Features
            </div>
            <div
              className={`nav-item ${currentPage === "visual" ? "active" : ""}`}
              onClick={() => setCurrentPage("visual")}
            >
              <span className="nav-icon">&#127912;</span>
              Visual Effects
            </div>
            <div
              className={`nav-item ${currentPage === "guides" ? "active" : ""}`}
              onClick={() => setCurrentPage("guides")}
            >
              <span className="nav-icon">&#128214;</span>
              Tool Guides
            </div>
          </div>
        </nav>

        <div className="sidebar-footer">
          <button
            className="quick-optimize-btn"
            onClick={handleQuickOptimize}
            disabled={loading}
          >
            {loading ? (
              <>
                <div className="spinner" style={{ width: 16, height: 16, borderWidth: 2 }}></div>
                Optimizing...
              </>
            ) : (
              <>
                <span>&#9889;</span>
                Quick Optimize
              </>
            )}
          </button>
        </div>
      </aside>

      <main className="main-content">
        {renderPage()}
      </main>

      {showModal && modalContent && (
        <div className="modal-overlay" onClick={() => setShowModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>{modalContent.title}</h3>
              <button className="modal-close" onClick={() => setShowModal(false)}>
                &times;
              </button>
            </div>
            <div className="modal-body">{modalContent.content}</div>
          </div>
        </div>
      )}

      <div className="toast-container">
        {toasts.map((toast) => (
          <div key={toast.id} className={`toast ${toast.type}`}>
            <span>
              {toast.type === "success" && "&#10004;"}
              {toast.type === "error" && "&#10006;"}
              {toast.type === "warning" && "&#9888;"}
            </span>
            {toast.message}
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;
