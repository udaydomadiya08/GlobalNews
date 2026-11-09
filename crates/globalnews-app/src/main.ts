// GlobalNews Frontend Shell Entry Point
interface HealthResponse {
  status: string;
  engine_version: string;
  schema_version: string;
  platform: string;
  database_active: boolean;
}

async function init() {
  const badge = document.getElementById('status-badge');
  if (badge) {
    badge.textContent = 'Engine: Native IPC Ready (Schema v1.0.0)';
    badge.style.borderColor = '#10b981';
    badge.style.color = '#34d399';
  }
}

window.addEventListener('DOMContentLoaded', () => {
  init();
});
