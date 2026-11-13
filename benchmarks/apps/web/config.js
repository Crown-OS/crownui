const params = new URLSearchParams(location.search);
const number = (name) => Number(params.get(name));
const benchConfig = {
  scenario: params.get('scenario'),
  width: number('width'),
  height: number('height'),
  rows: number('rows'),
  rowHeight: number('row_height'),
  swatch: number('swatch'),
  scrollSpeed: number('scroll_speed'),
  boxes: number('boxes'),
  boxSize: number('box_size'),
  labels: number('labels'),
  pages: number('pages'),
  sections: number('sections'),
  sectionRows: number('section_rows'),
  sidebarWidth: number('sidebar_width'),
  pageScrollSpeed: number('page_scroll_speed'),
  pageSeconds: number('page_seconds'),
};
