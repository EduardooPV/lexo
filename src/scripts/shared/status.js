export function setErrorStatus(node, message) {
  const prefix = document.createElement('span');
  prefix.className = 'sr-only';
  prefix.textContent = 'Error: ';
  node.classList.add('error');
  node.replaceChildren(prefix, message);
}
