document.documentElement.classList.add('js');
// Keep the current-page marker in the HTML for no-JS readers; this only repairs
// links when a static file is opened through an alternate pathname.
const current = location.pathname.split('/').pop() || 'index.html';
document.querySelectorAll('.chapter-menu a').forEach((link) => {
  if (link.getAttribute('href') === current) link.setAttribute('aria-current', 'page');
});
