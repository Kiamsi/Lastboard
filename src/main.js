import { invoke } from '@tauri-apps/api/core';

async function refreshUptime() {
  const info = await invoke('get_uptime');

  const date = new Date(info.time_system_started * 1000);
  const hours = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');

  document.getElementById('uptimeDisplay').textContent = info.uptime_formatted;
  document.getElementById('bootTimeDisplay').textContent = hours + ':' + minutes;
}

refreshUptime();
setInterval(refreshUptime, 1000);