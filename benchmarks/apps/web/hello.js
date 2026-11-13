function mountHello(root) {
  root.className = 'hello';
  const title = document.createElement('h1');
  title.textContent = 'Hello, world';
  const button = document.createElement('button');
  button.textContent = 'Click me';
  root.append(title, button);
}
