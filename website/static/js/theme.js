// gnaw docs — theme toggle, mobile nav, and basic search.
(function () {
  var root = document.documentElement;

  // ---- color theme --------------------------------------------------------
  function applyTheme(theme) {
    root.dataset.theme = theme;
    try { localStorage.setItem('theme', theme); } catch (e) { }
  }

  var toggle = document.querySelector('.theme-toggle');
  if (toggle) {
    toggle.addEventListener('click', function () {
      applyTheme(root.dataset.theme === 'dark' ? 'light' : 'dark');
    });
  }

  // ---- mobile navigation drawer -------------------------------------------
  var menuBtn = document.querySelector('.menu-btn');
  var scrim = document.querySelector('.scrim');
  function setNav(open) {
    document.body.classList.toggle('nav-open', open);
    if (menuBtn) menuBtn.setAttribute('aria-expanded', String(open));
  }
  if (menuBtn) menuBtn.addEventListener('click', function () {
    setNav(!document.body.classList.contains('nav-open'));
  });
  if (scrim) scrim.addEventListener('click', function () { setNav(false); });
  document.addEventListener('keydown', function (e) { if (e.key === 'Escape') setNav(false); });

  // ---- search (progressive enhancement) -----------------------------------
  // Works when Zola's search_index.en.js and elasticlunr are both present.
  var input = document.getElementById('search');
  var results = document.getElementById('search-results');
  if (!input || !results) return;

  var index = null;
  function buildIndex() {
    if (index) return index;
    if (typeof window.elasticlunr === 'undefined' || typeof window.searchIndex === 'undefined') return null;
    index = window.elasticlunr.Index.load(window.searchIndex);
    return index;
  }

  function render(items) {
    if (!items.length) { results.innerHTML = '<p class="sr-empty">No results</p>'; results.hidden = false; return; }
    results.innerHTML = items.map(function (r) {
      var doc = window.searchIndex.documents[r.ref];
      return '<a href="' + r.ref + '"><span class="sr-title">' + doc.title + '</span></a>';
    }).join('');
    results.hidden = false;
  }

  input.addEventListener('input', function () {
    var q = input.value.trim();
    if (q.length < 2) { results.hidden = true; return; }
    var idx = buildIndex();
    if (!idx) { results.innerHTML = '<p class="sr-empty">Search needs elasticlunr.min.js in static/js</p>'; results.hidden = false; return; }
    var hits = idx.search(q, { bool: 'AND', expand: true }).slice(0, 8);
    render(hits);
  });
  document.addEventListener('click', function (e) {
    if (!results.contains(e.target) && e.target !== input) results.hidden = true;
  });
})();

// ---- tabs ------------------------------------------------------------------
(function () {
  document.querySelectorAll('.tabs').forEach(function (tabs) {
    var btns = Array.prototype.slice.call(tabs.querySelectorAll(':scope > .tabs-list > .tab-btn'));
    var panels = Array.prototype.slice.call(tabs.querySelectorAll(':scope > .tab-panel'));
    function select(i) {
      btns.forEach(function (b, j) {
        b.classList.toggle('active', i === j);
        b.setAttribute('aria-selected', String(i === j));
        b.tabIndex = i === j ? 0 : -1;
      });
      panels.forEach(function (p, j) { p.hidden = i !== j; });
    }
    btns.forEach(function (b, i) {
      b.addEventListener('click', function () { select(i); });
      b.addEventListener('keydown', function (e) {
        if (e.key === 'ArrowRight') { select((i + 1) % btns.length); btns[(i + 1) % btns.length].focus(); }
        if (e.key === 'ArrowLeft') { select((i - 1 + btns.length) % btns.length); btns[(i - 1 + btns.length) % btns.length].focus(); }
      });
    });
    select(0);
  });
})();
