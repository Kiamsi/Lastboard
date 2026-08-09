import { invoke } from '@tauri-apps/api/core';

const bootTimeDisplay = document.getElementById("bootTimeDisplay");
const uptimeDisplay = document.getElementById("uptimeDisplay");
const processesDisplay = document.getElementById("processesDisplay");
const appCountDisplay = document.getElementById("app-count");
const networkSpeedDisplay = document.getElementById("networkSpeedDisplay");
const connectionsDisplay = document.getElementById("connectionsDisplay");
const osNameDisplay = document.getElementById("osNameDisplay");
const lanDevicesDisplay = document.getElementById("lanDevicesDisplay");
const listeningPortsDisplay = document.getElementById("listeningPortsDisplay");
const cpuUsageDisplay = document.getElementById("cpuUsageDisplay");
const cpuClockSpeedDisplay = document.getElementById("cpuClockSpeedDisplay");
const ramUsageDisplay = document.getElementById("ramUsageDisplay");
const diskIoDisplay = document.getElementById("diskIoDisplay");
const peripheralsDisplay = document.getElementById("peripheralsDisplay");
const osVersionDisplay = document.getElementById("osVersionDisplay");
const lastUpdateDisplay = document.getElementById("lastUpdateDisplay");
const monitorsDisplay = document.getElementById("monitorsDisplay");
const diskWriterTooltip = document.getElementById("diskWriterTooltip");
const lastProcessStarted = document.getElementById("lastProcessStarted");
const lastSystemSleepDisplay = document.getElementById("lastSystemSleep");
const lastFileDownloaded = document.getElementById("lastFileDownloaded");
const connectionsTooltip = document.getElementById("connectionsTooltip");
const lastInstalledAppDisplay = document.getElementById("lastInstalledApp");
const resetStatsButton = document.getElementById("resetStatsButton");


function formatUptime(totalSeconds) {
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    return `${hours}h ${minutes}m ${seconds}s`;
}

async function refreshUptime() {
    try {
        const { uptime } = await invoke('get_uptime');
        if (uptimeDisplay) uptimeDisplay.textContent = formatUptime(uptime);
    } catch (error) {
        console.error(error);
    }
}

async function refreshProcessCount() {
    try {
        const count = await invoke('get_process_count');
        if (processesDisplay) processesDisplay.textContent = count;
    } catch (error) {
        console.error(error);
    }
}

async function refreshAppCount() {
    try {
        const appsList = await invoke('get_installed_apps');
        if (appCountDisplay) appCountDisplay.textContent = appsList.length;
    } catch (error) {
        if (appCountDisplay) appCountDisplay.textContent = 'Error';
    }
}

async function refreshNetworkSpeed() {
    try {
        const [downBytes, upBytes] = await invoke('get_network_speed');
        const downMbps = ((downBytes * 8) / 1000000).toFixed(2);
        const upMbps = ((upBytes * 8) / 1000000).toFixed(2);
        if (networkSpeedDisplay) {
            networkSpeedDisplay.textContent = `${downMbps} ↓ / ${upMbps} ↑ Mbps`;
        }
    } catch (error) {
        if (networkSpeedDisplay) networkSpeedDisplay.textContent = 'Error';
    }
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

function renderConnectionLine(c) {
    const owner = c.process_name
        ? `${escapeHtml(c.process_name)} <span style="color:#6b737e;">(pid ${c.pid})</span>`
        : '<span style="color:#6b737e;">Unknown process</span>';
    const state = c.state || c.protocol;

    return `
        <div>
            <strong style="color: #c8ccd5;">${owner}</strong><br>
            ${c.protocol} ${c.local_address}:${c.local_port} &rarr;
            ${c.remote_address}:${c.remote_port}
            <span style="color: #939292; font-size: 0.9em;">(${state})</span>
        </div>
    `;
}

function renderConnectionGroup(connections, emptyText) {
    if (connections.length === 0) {
        return `<div style="color:#6b737e; font-style: italic;">${emptyText}</div>`;
    }
    return `<div style="display: flex; flex-direction: column; gap: 8px;">${connections.map(renderConnectionLine).join('')}</div>`;
}

async function refreshConnectionDetails() {
    if (!connectionsTooltip) return;
    try {
        const allConnections = await invoke('get_connections');
        const activeConnections = allConnections.filter(c => c.state === 'ESTABLISHED');
        const nonActiveConnections = allConnections.filter(c => c.state !== 'ESTABLISHED');

        if (connectionsDisplay) {
            connectionsDisplay.textContent = activeConnections.length;
        }

        connectionsTooltip.innerHTML =
            renderConnectionGroup(activeConnections, 'No active connections found.') +
            `<div class="connections-divider">Non-active (${nonActiveConnections.length})</div>` +
            renderConnectionGroup(nonActiveConnections, 'None.');

    } catch (error) {
        connectionsTooltip.innerHTML = 'Error loading connection data.';
    }
}

async function refreshLanDevices() {
    try {
        const count = await invoke('get_connected_lan_devices');
        if (lanDevicesDisplay) lanDevicesDisplay.textContent = count;
    } catch (error) {
        console.error(error);
    }
}

async function refreshListeningPorts() {
    try {
        const count = await invoke('get_listening_ports');
        if (listeningPortsDisplay) listeningPortsDisplay.textContent = count;
    } catch (error) {
        console.error(error);
    }
}

async function refreshCpuUsage() {
    try {
        const cpu = await invoke('get_cpu_usage');
        if (cpuUsageDisplay) {
            cpuUsageDisplay.textContent = `${cpu.toFixed(1)}%`;
        }
    } catch (error) {
        console.error(error);
    }
}

async function refreshCpuSpeed() {
    try {
        const speed = await invoke('get_cpu_speed');
        if (cpuClockSpeedDisplay) {
            cpuClockSpeedDisplay.textContent = `${(speed / 1000).toFixed(2)} GHz`;
        }
    } catch (error) {
        console.error(error);
    }
}

async function refreshRamUsage() {
    try {
        const [usedBytes, totalBytes] = await invoke('get_ram_usage');
        const usedGB = (usedBytes / 1073741824).toFixed(1);
        const totalGB = (totalBytes / 1073741824).toFixed(1);
        
        if (ramUsageDisplay) {
            ramUsageDisplay.textContent = `${usedGB} / ${totalGB} GB`;
        }
    } catch (error) {
        console.error(error);
    }
}

async function refreshDiskIo() {
  try {
    const [readBytes, writeBytes] = await invoke('get_disk_io');
    const readMb = (readBytes / 1048576).toFixed(2);
    const writeMb = (writeBytes / 1048576).toFixed(2);
    
    if (diskIoDisplay) {
        diskIoDisplay.textContent = `${readMb} R / ${writeMb} W MB/s`;
    }
  } catch (error) {
    console.error(error);
  }
}

async function refreshDiskWriter() {
    if (!diskWriterTooltip) return;
    try {
        const processInfo = await invoke('get_last_disk_writer');
        diskWriterTooltip.textContent = processInfo;
    } catch (error) {
        diskWriterTooltip.textContent = 'Error';
    }
}

async function refreshPeripherals() {
    try {
        const count = await invoke('get_connected_peripherals');
        if (peripheralsDisplay) peripheralsDisplay.textContent = count;
    } catch (error) {
        console.error(error);
    }
}

async function refreshMonitors() {
    try {
        const count = await invoke('get_monitors');
        if (monitorsDisplay) monitorsDisplay.textContent = count;
    } catch (error) {
        console.error(error);
    }
}

async function refreshLastProcessStarted() {
    try {
        const process = await invoke("get_last_started_process");
        if (lastProcessStarted) {
            lastProcessStarted.textContent = process;
        }
    } catch (error) {
        if (lastProcessStarted) 
            lastProcessStarted.textContent = "Error";
    }
}

async function refreshLastSleepTime() {
    try {
        const sleepTime = await invoke('get_last_sleep_time');
        if (lastSystemSleepDisplay) {
            lastSystemSleepDisplay.textContent = sleepTime;
        }
    } catch (error) {
        if (lastSystemSleepDisplay) {
            lastSystemSleepDisplay.textContent = 'Error';
        }
    }
}

async function refreshLastFileDownloaded() {
    try {
        const file = await invoke("get_last_downloaded_file");
        if (lastFileDownloaded) {
            lastFileDownloaded.textContent = file;
        }
    } catch (error) {
        if (lastFileDownloaded) {
            lastFileDownloaded.textContent = "Error";
        }
    }
}

async function refreshLastInstalledApp() {
    try {
        const app = await invoke("get_last_installed_app");
        if (lastInstalledAppDisplay) {
            lastInstalledAppDisplay.textContent = app;
        }
    } catch (error) {
        console.error("couldn't get the app", error);
        if (lastInstalledAppDisplay) {
            lastInstalledAppDisplay.textContent = "Error";
        }
    }
}

function refreshAllStats() {
    refreshUptime();
    refreshProcessCount();
    refreshAppCount();
    refreshNetworkSpeed();
    refreshConnectionDetails();
    refreshLanDevices();
    refreshListeningPorts();
    refreshCpuUsage();
    refreshCpuSpeed();
    refreshRamUsage();
    refreshDiskIo();
    refreshDiskWriter();
    refreshPeripherals();
    refreshMonitors();
    refreshLastProcessStarted();
    refreshLastSleepTime();
    refreshLastFileDownloaded();
    refreshLastInstalledApp();
}

async function initializeApp() {
    try {
        const initialData = await invoke('get_uptime');
        if (bootTimeDisplay) {
            bootTimeDisplay.textContent = new Date(initialData.time_system_started * 1000)
                .toTimeString()
                .slice(0, 5);
        }
    } catch (error) {
        console.error(error);
    }

    try {
        const osName = await invoke('get_os_name');
        if (osNameDisplay) {
            osNameDisplay.textContent = osName;
        }
    } catch (error) {
        console.error(error);
    }

    try {
        const osVersion = await invoke('get_os_version');
        if (osVersionDisplay) {
            osVersionDisplay.textContent = osVersion;
        }
    } catch (error) {
        console.error(error);
    }

    try {
        const updateTimestamp = await invoke('get_last_system_update');
        if (lastUpdateDisplay) {
            if (!updateTimestamp) {
                lastUpdateDisplay.textContent = 'Unknown';
            } else {
                const updateDate = new Date(updateTimestamp * 1000);
                const year = updateDate.getFullYear();
                const month = String(updateDate.getMonth() + 1).padStart(2, '0');
                const day = String(updateDate.getDate()).padStart(2, '0');
                lastUpdateDisplay.textContent = `${year}-${month}-${day}`;
            }
        }
    } catch (error) {
        console.error(error);
    }

    refreshAllStats();

    if (resetStatsButton) {
        resetStatsButton.addEventListener('click', () => {
            refreshAllStats();
            resetStatsButton.classList.remove('spinning');
            void resetStatsButton.offsetWidth;
            resetStatsButton.classList.add('spinning');
        });
    }

    setInterval(refreshUptime, 1000);
    setInterval(refreshProcessCount, 1000);
    setInterval(refreshNetworkSpeed, 1000); 
    setInterval(refreshConnectionDetails, 2500);
    setInterval(refreshAppCount, 300_000); 
    setInterval(refreshLanDevices, 60_000);
    setInterval(refreshListeningPorts, 5000);
    setInterval(refreshCpuUsage, 1000);
    setInterval(refreshCpuSpeed, 1000);
    setInterval(refreshRamUsage, 4000);
    setInterval(refreshDiskIo, 1000);
    setInterval(refreshDiskWriter, 1000);
    setInterval(refreshPeripherals, 60_000);
    setInterval(refreshMonitors, 180_000);
    setInterval(refreshLastProcessStarted, 1000);
    setInterval(refreshLastSleepTime, 600_000);
    setInterval(refreshLastFileDownloaded, 10_000);
    setInterval(refreshLastInstalledApp, 300_000);
}

window.addEventListener('DOMContentLoaded', initializeApp);