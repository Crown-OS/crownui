const { app, BrowserWindow, Menu } = require('electron');
const path = require('node:path');

const query = Object.fromEntries(
  Object.entries(process.env)
    .filter(([name]) => name.startsWith('BENCH_'))
    .map(([name, value]) => [name.slice('BENCH_'.length).toLowerCase(), value]),
);

Menu.setApplicationMenu(null);
app.whenReady().then(() => {
  const window = new BrowserWindow({
    width: Number(query.width),
    height: Number(query.height),
    useContentSize: true,
    title: 'Electron bench',
    backgroundColor: '#f8fafc',
  });
  window.loadFile(path.join(__dirname, '..', 'web', 'index.html'), { query });
});
app.on('window-all-closed', () => app.quit());
