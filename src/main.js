import { invoke } from '@tauri-apps/api/core';

const bootTimeDisplay = document.getElementById('bootTimeDisplay');
const uptimeDisplay = document.getElementById('uptimeDisplay');
const processesDisplay = document.getElementById('processesDisplay');

function formatUptime(totalSeconds) {

  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return `${hours}h ${minutes}m ${seconds}s`;

}

const initialData = await invoke('get_uptime');

bootTimeDisplay.textContent = new Date(initialData.time_system_started * 1000)
  .toTimeString()
  .slice(0, 5);

async function refreshUptime() {

  const { uptime } = await invoke('get_uptime');
  uptimeDisplay.textContent = formatUptime(uptime);

}

async function refreshProcessCount() {
  const count = await invoke('get_process_count');
  processesDisplay.textContent = count;
}

refreshUptime();
refreshProcessCount();
setInterval(refreshUptime, 1000);
setInterval(refreshProcessCount, 1000);