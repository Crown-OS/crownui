function listRow(index, config) {
  const row = document.createElement('div');
  row.className = 'row';
  row.style.height = `${config.rowHeight}px`;
  const swatch = document.createElement('div');
  swatch.className = 'swatch';
  swatch.style.width = swatch.style.height = `${config.swatch}px`;
  swatch.style.background = swatchColor(index);
  const title = document.createElement('div');
  title.className = 'title';
  title.textContent = `Row ${index}`;
  const subtitle = document.createElement('div');
  subtitle.className = 'subtitle';
  subtitle.textContent = `Subtitle for row ${index}`;
  const text = document.createElement('div');
  text.append(title, subtitle);
  row.append(swatch, text);
  return row;
}

function mountList(root, config) {
  root.className = 'list';
  const rows = document.createDocumentFragment();
  for (let index = 0; index < config.rows; index++) rows.append(listRow(index, config));
  root.append(rows);
  const travel = Math.max(1, config.rows * config.rowHeight - config.height);
  let started;
  const scroll = (now) => {
    started ??= now;
    root.scrollTop = (((now - started) / 1000) * config.scrollSpeed) % travel;
    requestAnimationFrame(scroll);
  };
  requestAnimationFrame(scroll);
}
