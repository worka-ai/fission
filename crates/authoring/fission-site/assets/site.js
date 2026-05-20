(() => {
  const key = 'fission-site-theme';
  const root = document.documentElement;
  const apply = (theme) => {
    root.dataset.theme = theme;
    try { localStorage.setItem(key, theme); } catch (_) {}
  };
  let current = 'light';
  try { current = localStorage.getItem(key) || (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'); } catch (_) {}
  apply(current);
  document.addEventListener('click', (event) => {
    const button = event.target.closest('[data-theme-toggle]');
    if (!button) return;
    apply(root.dataset.theme === 'dark' ? 'light' : 'dark');
  });
})();
