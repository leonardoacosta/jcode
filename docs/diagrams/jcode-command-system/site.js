document.documentElement.classList.add('js');
const current = location.pathname.split('/').pop() || 'index.html';
document.querySelectorAll('.chapter-menu a').forEach((link) => {
  if (link.getAttribute('href') === current) link.setAttribute('aria-current', 'page');
});
