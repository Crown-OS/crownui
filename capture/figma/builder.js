// Rebuilds captured CrownUI scenes as editable Figma layers.
// Runs inside use_figma as `new AsyncFunction('figma', 'chunks', 'target', builder)`, where
// `chunks` maps PNG chunk types to Uint8Array lists and `target` is
// { pageId, only?: [state names], x?, y? }.

const BLUR_SCALE = 2;
const TEXT = { r: 0.94, g: 0.94, b: 0.96 };
const MUTED = { r: 0.6, g: 0.61, b: 0.66 };
const SECTION_FILL = { r: 0.09, g: 0.09, b: 0.11 };
const WEIGHTS = [
  [100, 'Thin'], [200, 'Extra Light'], [300, 'Light'], [400, 'Regular'], [500, 'Medium'],
  [600, 'Semi Bold'], [700, 'Bold'], [800, 'Extra Bold'], [900, 'Black'],
];

const ascii = (bytes) => {
  let text = '';
  for (let i = 0; i < bytes.length; i += 8192) text += String.fromCharCode.apply(null, bytes.subarray(i, i + 8192));
  return text;
};
const hexKey = (bytes) => Array.from(bytes.subarray(0, 8), (b) => b.toString(16).padStart(2, '0')).join('');

const page = JSON.parse(ascii(chunks.crPg[0]));
const scenes = (chunks.crSc || []).map((bytes) => JSON.parse(ascii(bytes)));
const imageBytes = new Map((chunks.crIm || []).map((bytes) => [hexKey(bytes), bytes.subarray(8)]));
const imageHashes = new Map();
const baseName = (scene) => scene.name.replace(/ · (Light|Dark)$/, '');
const wanted = target.only ? scenes.filter((scene) => target.only.includes(baseName(scene))) : scenes;

const styleOf = (weight, italic) => {
  const name = WEIGHTS.reduce((best, entry) => (Math.abs(entry[0] - weight) < Math.abs(best[0] - weight) ? entry : best))[1];
  if (!italic) return name;
  return name === 'Regular' ? 'Italic' : `${name} Italic`;
};
const fonts = new Set(['Regular', 'Medium', 'Semi Bold', 'Bold']);
const collectFonts = (items) => {
  for (const item of items) {
    if (item.type === 'text') fonts.add(styleOf(item.weight, item.italic));
    if (item.items) collectFonts(item.items);
  }
};
for (const scene of wanted) collectFonts(scene.root.items);
await Promise.all([...fonts].map((style) => figma.loadFontAsync({ family: 'Inter', style })));

const rgba = (hex) => ({
  r: parseInt(hex.slice(0, 2), 16) / 255,
  g: parseInt(hex.slice(2, 4), 16) / 255,
  b: parseInt(hex.slice(4, 6), 16) / 255,
  a: parseInt(hex.slice(6, 8), 16) / 255,
});
const solid = (hex) => {
  const { r, g, b, a } = rgba(hex);
  return { type: 'SOLID', color: { r, g, b }, opacity: a };
};
const imageHash = (key) => {
  const id = key;
  if (!imageHashes.has(id) && imageBytes.has(id)) imageHashes.set(id, figma.createImage(imageBytes.get(id)).hash);
  return imageHashes.get(id);
};
const invert = ([[a, c, e], [b, d, f]]) => {
  const det = a * d - b * c || 1e-6;
  return [[d / det, -c / det, (c * f - d * e) / det], [-b / det, a / det, (b * e - a * f) / det]];
};
const paint = (fill, box) => {
  if (fill.type === 'solid') return solid(fill.color);
  if (fill.type === 'image') {
    const hash = imageHash(fill.image);
    return hash ? { type: 'IMAGE', imageHash: hash, scaleMode: 'FILL' } : solid('00000000');
  }
  const [x, y, w, h] = box;
  const width = w || 1;
  const height = h || 1;
  const dx = fill.end[0] - fill.start[0];
  const dy = fill.end[1] - fill.start[1];
  const ox = (fill.start[0] - x) / width + (0.5 * dy) / width;
  const oy = (fill.start[1] - y) / height - (0.5 * dx) / height;
  return {
    type: 'GRADIENT_LINEAR',
    gradientTransform: invert([[dx / width, -dy / width, ox], [dy / height, dx / height, oy]]),
    gradientStops: fill.stops.map(([position, hex]) => ({ position: Math.min(1, Math.max(0, position)), color: rgba(hex) })),
  };
};
const blurOf = (fill) => (fill && fill.blur ? [{ type: 'LAYER_BLUR', radius: fill.blur * BLUR_SCALE, visible: true }] : []);
const shadowEffects = (shadows) => (shadows || []).map((shadow) => ({
  type: 'DROP_SHADOW', color: rgba(shadow.color), offset: { x: shadow.offset[0], y: shadow.offset[1] },
  radius: shadow.blur, spread: 0, visible: true, blendMode: 'NORMAL', showShadowBehindNode: false,
}));
const setRadii = (node, radii, smoothing) => {
  const [tl, tr, br, bl] = radii;
  if (tl === tr && tr === br && br === bl) node.cornerRadius = tl;
  else Object.assign(node, { topLeftRadius: tl, topRightRadius: tr, bottomRightRadius: br, bottomLeftRadius: bl });
  if (tl || tr || br || bl) node.cornerSmoothing = smoothing;
};
const setStroke = (node, stroke, box) => {
  if (!stroke) return;
  node.strokes = [paint(stroke.fill, box)];
  node.strokeWeight = stroke.width;
  node.strokeAlign = 'CENTER';
  node.strokeCap = { butt: 'NONE', round: 'ROUND', square: 'SQUARE' }[stroke.cap];
  node.strokeJoin = stroke.join.toUpperCase();
};
const size = (node, width, height) => node.resize(Math.max(width, 0.01), Math.max(height, 0.01));
const place = (node, [a, b, c, d, tx, ty], ox, oy) => {
  if (a === 1 && b === 0 && c === 0 && d === 1) {
    node.x = tx - ox;
    node.y = ty - oy;
    return;
  }
  const scaleX = Math.hypot(a, b);
  const scaleY = Math.hypot(c, d);
  if (Math.abs(scaleX - 1) > 1e-3 || Math.abs(scaleY - 1) > 1e-3) size(node, node.width * scaleX, node.height * scaleY);
  node.relativeTransform = [[a / scaleX, c / scaleY, tx - ox], [b / scaleX, d / scaleY, ty - oy]];
};

function buildItems(items, parent, ox, oy, smoothing) {
  for (const item of items) buildItem(item, parent, ox, oy, smoothing);
}

function buildNode(node, parent, ox, oy, smoothing) {
  const frame = figma.createFrame();
  parent.appendChild(frame);
  frame.name = node.name || 'view';
  size(frame, node.width, node.height);
  place(frame, node.matrix, ox, oy);
  frame.fills = node.fill ? [paint(node.fill, [0, 0, node.width, node.height])] : [];
  frame.clipsContent = node.clip;
  if (node.opacity !== 1) frame.opacity = node.opacity;
  setRadii(frame, node.radii, smoothing);
  setStroke(frame, node.stroke, [0, 0, node.width, node.height]);
  const effects = [...shadowEffects(node.shadows), ...blurOf(node.fill)];
  if (node.backdrop_blur) effects.push({ type: 'BACKGROUND_BLUR', radius: node.backdrop_blur * BLUR_SCALE, visible: true });
  if (effects.length) frame.effects = effects;
  buildItems(node.items, frame, 0, 0, smoothing);
  return frame;
}

function buildItem(item, parent, ox, oy, smoothing) {
  if (item.type === 'node') return buildNode(item, parent, ox, oy, smoothing);
  if (item.type === 'group') {
    const frame = figma.createFrame();
    parent.appendChild(frame);
    frame.name = item.name;
    frame.fills = [];
    const [cx, cy, cw, ch] = item.clip || [0, 0, parent.width, parent.height];
    size(frame, cw, ch);
    place(frame, [...item.matrix.slice(0, 4), item.matrix[4] + cx, item.matrix[5] + cy], ox, oy);
    frame.clipsContent = !!item.clip;
    if (item.clip) setRadii(frame, item.radii, smoothing);
    if (item.opacity !== 1) frame.opacity = item.opacity;
    if (item.backdrop_blur) frame.effects = [{ type: 'BACKGROUND_BLUR', radius: item.backdrop_blur * BLUR_SCALE, visible: true }];
    buildItems(item.items, frame, cx, cy, smoothing);
    return frame;
  }
  if (item.type === 'rect') {
    const [x, y, w, h] = item.rect;
    const rect = figma.createRectangle();
    parent.appendChild(rect);
    size(rect, w, h);
    rect.x = x - ox;
    rect.y = y - oy;
    setRadii(rect, item.radii, smoothing);
    if (item.fill) {
      rect.name = item.fill.type === 'image' ? 'image' : 'fill';
      rect.fills = [paint(item.fill, item.rect)];
      setStroke(rect, item.stroke, item.rect);
      const effects = [...shadowEffects(item.shadows), ...blurOf(item.fill)];
      if (effects.length) rect.effects = effects;
    } else if (item.stroke) {
      rect.name = 'outline';
      rect.fills = [];
      setStroke(rect, item.stroke, item.rect);
    } else {
      const shadow = item.shadows[0];
      rect.name = 'shadow';
      rect.fills = [solid(shadow.color)];
      rect.x += shadow.offset[0];
      rect.y += shadow.offset[1];
      rect.effects = [{ type: 'LAYER_BLUR', radius: shadow.blur, visible: true }];
    }
    return rect;
  }
  if (item.type === 'path') {
    const vector = figma.createVector();
    parent.appendChild(vector);
    vector.name = item.stroke && !item.fill ? 'stroke' : 'shape';
    vector.x = -ox;
    vector.y = -oy;
    vector.vectorPaths = [{ windingRule: 'NONZERO', data: item.data }];
    const box = [vector.x + ox, vector.y + oy, vector.width, vector.height];
    vector.fills = item.fill ? [paint(item.fill, box)] : [];
    if (item.stroke) setStroke(vector, item.stroke, box);
    const effects = blurOf(item.fill);
    if (effects.length) vector.effects = effects;
    return vector;
  }
  if (item.type === 'image') {
    const [x, y, w, h] = item.rect;
    const rect = figma.createRectangle();
    parent.appendChild(rect);
    rect.name = 'image';
    size(rect, w, h);
    rect.x = x - ox;
    rect.y = y - oy;
    const hash = imageHash(item.image);
    rect.fills = hash ? [{ type: 'IMAGE', imageHash: hash, scaleMode: 'FILL' }] : [];
    return rect;
  }
  if (item.type === 'text') return buildText(item, parent, ox, oy);
  return null;
}

function buildText(item, parent, ox, oy) {
  const text = figma.createText();
  parent.appendChild(text);
  text.fontName = { family: 'Inter', style: styleOf(item.weight, item.italic) };
  text.characters = item.text;
  text.name = item.text.split('\n')[0].slice(0, 48) || 'text';
  text.fontSize = item.size;
  text.fills = [solid(item.color)];
  text.lineHeight = { unit: 'PIXELS', value: item.height / Math.max(item.lines, 1) };
  if (item.letter_spacing) text.letterSpacing = { unit: 'PIXELS', value: item.letter_spacing };
  text.textAlignHorizontal = { start: 'LEFT', center: 'CENTER', end: 'RIGHT', justify: 'JUSTIFIED' }[item.align];
  const slack = item.align === 'center' ? 1 : item.align === 'end' ? 2 : 0;
  if (item.max_lines) {
    text.textAutoResize = 'NONE';
    size(text, item.width + 2, item.height);
    text.textTruncation = 'ENDING';
    text.maxLines = item.max_lines;
  } else if (item.lines > 1 || item.align !== 'start') {
    text.textAutoResize = 'HEIGHT';
    size(text, Math.max(item.wrap_width && item.lines > 1 ? item.wrap_width : item.width, item.width) + 2, item.height);
  } else {
    text.textAutoResize = 'WIDTH_AND_HEIGHT';
  }
  if (item.matrix) place(text, [...item.matrix.slice(0, 4), item.matrix[4] + item.x, item.matrix[5] + item.y], ox, oy);
  else {
    text.x = item.x - ox - slack;
    text.y = item.y - oy;
  }
  if (item.blur) text.effects = [{ type: 'LAYER_BLUR', radius: item.blur * BLUR_SCALE, visible: true }];
  return text;
}

function buildScene(scene) {
  const frame = figma.createFrame();
  frame.name = scene.name;
  size(frame, scene.width, scene.height);
  const clear = rgba(scene.clear);
  const fills = [];
  const material = scene.material;
  if (material && material.tint) fills.push(solid(material.tint));
  if (clear.a > 0) fills.push(solid(scene.clear));
  frame.fills = fills;
  frame.clipsContent = true;
  if (material) {
    frame.cornerRadius = material.corner_radius;
    frame.cornerSmoothing = scene.corner_smoothing;
    const effects = shadowEffects(material.shadow ? [material.shadow] : []);
    if (material.blur) effects.push({ type: 'BACKGROUND_BLUR', radius: material.blur * BLUR_SCALE, visible: true });
    frame.effects = effects;
  }
  buildItems(scene.root.items, frame, 0, 0, scene.corner_smoothing);
  return frame;
}

const label = (content, style, fontSize, color, width) => {
  const text = figma.createText();
  text.fontName = { family: 'Inter', style };
  text.characters = content;
  text.fontSize = fontSize;
  text.fills = [{ type: 'SOLID', color }];
  if (width) {
    text.textAutoResize = 'HEIGHT';
    text.resize(width, text.height);
  }
  text.name = content.slice(0, 60);
  return text;
};
const column = (name, gap) => figma.createAutoLayout('VERTICAL', { name, itemSpacing: gap, fills: [] });
const row = (name, gap) => figma.createAutoLayout('HORIZONTAL', { name, itemSpacing: gap, fills: [] });
const childNamed = (parent, name, type) => parent.children.find((child) => child.name === name && (!type || child.type === type));
const pageBottom = (node) => node.children.reduce((bottom, child) => Math.max(bottom, child.y + child.height), 0);

const pageNode = await figma.getNodeByIdAsync(target.pageId);
await figma.setCurrentPageAsync(pageNode);
if (!pageNode.children.length) {
  const title = column('Title', 24);
  pageNode.appendChild(title);
  title.appendChild(label(page.app, 'Bold', 120, TEXT));
  title.appendChild(label(page.description, 'Regular', 40, MUTED, 4000));
}

const groups = [...new Set(wanted.map((scene) => scene.group))];
const built = [];
for (const group of groups) {
  let section = childNamed(pageNode, group, 'SECTION');
  if (!section) {
    const top = pageBottom(pageNode) + 200;
    section = figma.createSection();
    pageNode.appendChild(section);
    section.name = group;
    section.x = 0;
    section.y = top;
    section.fills = [{ type: 'SOLID', color: SECTION_FILL }];
  }
  let content = childNamed(section, 'Content', 'FRAME');
  if (!content) {
    content = column('Content', 96);
    section.appendChild(content);
    content.x = 0;
    content.y = 0;
    content.paddingTop = content.paddingBottom = content.paddingLeft = content.paddingRight = 160;
    const heading = column('Heading', 16);
    content.appendChild(heading);
    heading.appendChild(label(group, 'Bold', 88, TEXT));
    heading.appendChild(label(`${page.app} · ${group} — light and dark, captured from the current build.`, 'Regular', 36, MUTED));
  }
  for (const theme of ['Light', 'Dark']) {
    const sceneList = wanted.filter((scene) => scene.group === group && scene.theme === theme);
    if (!sceneList.length) continue;
    let faces = childNamed(content, theme, 'FRAME');
    if (!faces) {
      faces = row(theme, 160);
      content.appendChild(faces);
    }
    for (const scene of sceneList) {
      const state = baseName(scene);
      const old = childNamed(faces, state, 'FRAME');
      const face = column(state, 40);
      if (old) {
        faces.insertChild(faces.children.indexOf(old), face);
        old.remove();
      } else faces.appendChild(face);
      const window = buildScene(scene);
      face.appendChild(window);
      const caption = column('Caption', 12);
      face.appendChild(caption);
      caption.appendChild(label(state, 'Semi Bold', 56, TEXT));
      caption.appendChild(label(`${theme} · ${scene.width}×${scene.height}${scene.description ? ` · ${scene.description}` : ''}`, 'Medium', 34, MUTED, Math.max(scene.width, 600)));
      built.push({ name: scene.name, id: window.id });
    }
  }
  section.resizeWithoutConstraints(content.width, content.height);
}
const ownGroups = new Set(scenes.map((scene) => scene.group));
const own = pageNode.children
  .filter((node) => node.type === 'SECTION' && ownGroups.has(node.name))
  .sort((first, second) => first.y - second.y);
let y = own.length ? own[0].y : 0;
for (const section of own) {
  section.y = y;
  y += section.height + 200;
}
return { page: pageNode.name, built, scenes: scenes.length };
