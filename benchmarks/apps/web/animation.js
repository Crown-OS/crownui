const cellFactor = 1.5;
const wobble = 6;
const labelColumns = 20;
const labelRowHeight = 18;
const labelGap = 8;

function mountAnimation(root, config) {
  root.className = 'stage';
  const cell = config.boxSize * cellFactor;
  const columns = Math.max(1, Math.floor(config.width / cell));
  const inset = (cell - config.boxSize) / 2;
  const boxes = Array.from({ length: config.boxes }, (_, index) => {
    const box = document.createElement('div');
    box.className = 'box';
    box.style.left = `${(index % columns) * cell + inset}px`;
    box.style.top = `${Math.floor(index / columns) * cell + inset}px`;
    box.style.width = box.style.height = `${config.boxSize}px`;
    box.style.background = swatchColor(index);
    return box;
  });
  const labelWidth = config.width / labelColumns;
  const labelTop = Math.ceil(config.boxes / columns) * cell + labelGap;
  const labels = Array.from({ length: config.labels }, (_, index) => {
    const label = document.createElement('span');
    label.className = 'label';
    label.style.left = `${(index % labelColumns) * labelWidth}px`;
    label.style.top = `${labelTop + Math.floor(index / labelColumns) * labelRowHeight}px`;
    return label;
  });
  root.append(...boxes, ...labels);
  let started;
  const frame = (now) => {
    started ??= now;
    const seconds = (now - started) / 1000;
    boxes.forEach((box, index) => {
      const phase = seconds * 2 + index * 0.15;
      const rotation = (seconds * 90 + index * 3) % 360;
      box.style.transform = `translate(${wobble * Math.sin(phase)}px, ${wobble * Math.cos(phase)}px) rotate(${rotation}deg)`;
    });
    labels.forEach((label, index) => {
      label.textContent = String(Math.floor(seconds * 1000) + index);
    });
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}
