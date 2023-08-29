
async function getSession() {
  const result = await fetch('/session', { credentials: 'include' });
  const data = await result.json();
  return data.session;
}

async function postSession(session) {
  const result = await fetch('/session', {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session })
  });
  const data = await result.json();
  return data.session;
}

async function listLinks() {
  const result = await fetch('/list', { credentials: 'include' });

  const data = await result.json();
  return data.links
}

async function save(url, token) {
  const result = await fetch('/save', {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url, token })
  });
  const data = await result.json();
  return data.token;
}

async function forget(token) {
  const result = await fetch('/forget', {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ token })
  });
  const data = await result.json();
  return data.token;
}

document.addEventListener('alpine:init', () => {
  Alpine.store('smrs', {
    session: null,
    nextAlertId: 0,
    alerts: [],

    loadSession() {
      this.session = getSession();
    },
    addAlert(msg, type = 'success') {
      this.alerts.push({ id: this.nextAlertId++, msg, type });
    },
    removeAlert(id) {
      this.alerts = this.alerts.filter(alert => alert.id !== id);
    },
  })
})

