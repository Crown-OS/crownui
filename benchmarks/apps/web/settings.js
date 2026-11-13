// The settings scenario, by the rules in apps/shared/settings.rs.
const settingsPages = [
  ['Wi-Fi', 'wifi'],
  ['Bluetooth', 'bluetooth'],
  ['Network', 'globe'],
  ['Display', 'monitor'],
  ['Sound', 'volume-2'],
  ['Notifications', 'bell'],
  ['Battery & Power', 'battery-full'],
  ['Appearance', 'palette'],
];

const pageTitle = (page) => settingsPages[page % settingsPages.length][0];
const pageIcon = (page) => settingsPages[page % settingsPages.length][1];

function element(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function icon(name) {
  const image = element('img', 'icon');
  image.src = `icons/${name}.svg`;
  return image;
}

function settingsControl(page, index) {
  switch ((page + index) % 4) {
    case 0: {
      const track = element('div', (page + index) % 3 === 0 ? 'switch on' : 'switch');
      track.append(element('div', 'knob'));
      return track;
    }
    case 1: {
      const slider = element('input', 'slider');
      slider.type = 'range';
      slider.value = String((page * 7 + index * 13) % 100);
      return slider;
    }
    case 2: {
      const value = element('div', 'value');
      value.append(element('span', '', 'Automatic'), icon('chevron-right'));
      return value;
    }
    default:
      return icon('chevron-right');
  }
}

function settingsRow(page, index) {
  const row = element('div', 'setting');
  const label = element('div', 'setting-label');
  label.append(
    element('div', 'setting-title', `${pageTitle(page)} option ${index + 1}`),
    element('div', 'setting-description', `Describes what option ${index + 1} changes`),
  );
  row.append(label, settingsControl(page, index));
  return row;
}

function settingsPage(page, config) {
  const scroller = element('div', 'page');
  const content = element('div', 'page-content');
  content.append(element('h1', 'page-title', pageTitle(page)));
  for (let section = 0; section < config.sections; section++) {
    const group = element('section', 'group');
    const card = element('div', 'card');
    for (let row = 0; row < config.sectionRows; row++) {
      card.append(settingsRow(page, section * config.sectionRows + row));
    }
    group.append(element('div', 'group-title', `SECTION ${section + 1}`), card);
    content.append(group);
  }
  scroller.append(content);
  return scroller;
}

function mountSettings(root, config) {
  root.className = 'settings';
  const sidebar = element('nav', 'sidebar');
  sidebar.style.width = `${config.sidebarWidth}px`;
  sidebar.append(element('div', 'sidebar-title', 'Settings'));
  const items = [];
  for (let page = 0; page < config.pages; page++) {
    const item = element('div', 'sidebar-item');
    item.append(icon(pageIcon(page)), element('span', '', pageTitle(page)));
    items.push(item);
  }
  sidebar.append(...items);
  const pane = element('main', 'pane');
  root.append(sidebar, pane);

  let started;
  let open = -1;
  let scroller;
  const frame = (now) => {
    started ??= now;
    const seconds = (now - started) / 1000;
    const opened = Math.floor(seconds / config.pageSeconds);
    const page = opened % config.pages;
    if (page !== open) {
      items[open]?.classList.remove('selected');
      items[page].classList.add('selected');
      scroller = settingsPage(page, config);
      pane.replaceChildren(scroller);
      open = page;
    }
    const travel = Math.max(1, scroller.scrollHeight - scroller.clientHeight);
    scroller.scrollTop = ((seconds - opened * config.pageSeconds) * config.pageScrollSpeed) % travel;
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}
