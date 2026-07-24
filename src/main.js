import { invoke } from '@tauri-apps/api/core';

const bootTimeDisplay = document.getElementById('bootTimeDisplay');
const uptimeDisplay = document.getElementById('uptimeDisplay');
const processesDisplay = document.getElementById('processesDisplay');
const appCountDisplay = document.getElementById('app-count');
const networkSpeedDisplay = document.getElementById('networkSpeedDisplay');
const connectionsDisplay = document.getElementById('connectionsDisplay');
const osNameDisplay = document.getElementById('osNameDisplay');
const lanDevicesDisplay = document.getElementById('lanDevicesDisplay');
const listeningPortsDisplay = document.getElementById('listeningPortsDisplay');

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
        console.error('Failed to fetch uptime:', error);
    }
}

async function refreshProcessCount() {
    try {
        const count = await invoke('get_process_count');
        if (processesDisplay) processesDisplay.textContent = count;
    } catch (error) {
        console.error('Failed to fetch process count:', error);
    }
}

async function refreshAppCount() {
    try {
        const appsList = await invoke('get_installed_apps');
        if (appCountDisplay) appCountDisplay.textContent = appsList.length;
    } catch (error) {
        console.error('Failed to fetch app count:', error);
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
        console.error('Failed to fetch network speed:', error);
        if (networkSpeedDisplay) networkSpeedDisplay.textContent = 'Error';
    }
}

async function refreshOpenConnections() {
    try {
        const count = await invoke('get_open_connections');
        if (connectionsDisplay) connectionsDisplay.textContent = count;
    } catch (error) {
        console.error('Failed to fetch open connections:', error);
    }
}

async function refreshLanDevices() {
    try {
        const count = await invoke('get_connected_lan_devices');
        if (lanDevicesDisplay) lanDevicesDisplay.textContent = count;
    } catch (error) {
        console.error('Failed to fetch LAN devices count:', error);
    }
}

async function refreshListeningPorts() {
    try {
        const count = await invoke('get_listening_ports');
        if (listeningPortsDisplay) listeningPortsDisplay.textContent = count;
    } catch (error) {
        console.error('Failed to fetch listening ports:', error);
    }
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
        console.error('Failed to fetch boot time:', error);
    }

    try {
        const osName = await invoke('get_os_name');
        if (osNameDisplay) {
            osNameDisplay.textContent = osName;
        }
    } catch (error) {
        console.error('Failed to fetch OS name:', error);
    }

    refreshUptime();
    refreshProcessCount();
    refreshAppCount();
    refreshNetworkSpeed();
    refreshOpenConnections();
    refreshLanDevices();
    refreshListeningPorts();

    setInterval(refreshUptime, 1000);
    setInterval(refreshProcessCount, 1000);
    setInterval(refreshNetworkSpeed, 1000); 
    setInterval(refreshOpenConnections, 2500);
    setInterval(refreshAppCount, 300000); 
    setInterval(refreshLanDevices, 60000);
    setInterval(refreshListeningPorts, 5000);
}

window.addEventListener('DOMContentLoaded', initializeApp);