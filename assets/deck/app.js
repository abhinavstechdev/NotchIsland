'use strict';

(() => {
  const TOKEN_KEY = 'notchpixel-deck-token';
  const PAGE_KEY = 'notchpixel-deck-page';
  const MODE_KEY = 'notchpixel-deck-mode';
  const HOME_TIP_KEY = 'notchpixel-deck-home-hint';
  const POLL_MS = 1500;
  const PAIR_POLL_MS = 1000;
  const RETRY_MS = 2500;
  const OFFLINE_AFTER = 2;
  const SEND_LEVEL_MS = 70;
  const PAGES = [
    { id: 'now', label: 'Now' },
    { id: 'pad', label: 'Pad' },
    { id: 'apps', label: 'Apps' },
    { id: 'modes', label: 'Modes' },
    { id: 'pc', label: 'PC' },
  ];
  const MODES = [
    { id: 'game', label: 'Game' },
    { id: 'present', label: 'Present' },
    { id: 'movie', label: 'Movie' },
    { id: 'work', label: 'Work' },
  ];
  const DONE = {
    'game:shot': 'Screenshot saved', 'game:clip': 'Clip saved', 'game:record': 'Recording switched',
    'present:next': 'Next slide', 'present:prev': 'Back a slide', 'present:start': 'Presenting',
    'movie:full': 'Full screen', 'work:copy': 'Copied', 'work:paste': 'Pasted', 'work:undo': 'Undone', 'work:snip': 'Snip',
    'sys:lock': 'Locked', 'sys:sleep': 'Going to sleep', 'sys:restart': 'Restarting in 30 s', 'sys:shutdown': 'Shutting down in 30 s',
  };

  const $ = (id) => document.getElementById(id);
  let token = null;
  let pcName = 'PC';
  let misses = 0;
  let meterViews = [];
  let artVersion = -1;
  let artUrl = null;
  let draggingVolume = false;
  let draggingSeek = false;
  let micMuted = false;

  const readToken = () => { try { return localStorage.getItem(TOKEN_KEY); } catch { return null; } };
  const saveToken = (value) => { try { localStorage.setItem(TOKEN_KEY, value); } catch { /* it lasts this visit only */ } };
  const forgetToken = () => { try { localStorage.removeItem(TOKEN_KEY); } catch { /* nothing to forget */ } };
  const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  async function post(path, body) {
    const response = await fetch(path, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
    const payload = await response.json();
    return { status: response.status, data: payload.data, error: payload.error };
  }

  async function api(path, body) {
    const options = { headers: { Authorization: `Bearer ${token}` } };
    if (body !== undefined) {
      options.method = 'POST';
      options.headers['Content-Type'] = 'application/json';
      options.body = JSON.stringify(body);
    }
    const response = await fetch(path, options);
    const payload = await response.json();
    if (response.status === 409) setBlocked(true);
    if (!response.ok || !payload.success) {
      const error = new Error(payload.error || 'Request failed');
      error.status = response.status;
      throw error;
    }
    return payload.data;
  }

  async function blob(path) {
    const response = await fetch(path, { headers: { Authorization: `Bearer ${token}` } });
    if (!response.ok) return null;
    return URL.createObjectURL(await response.blob());
  }

  async function runAction(action) {
    try {
      await api('/api/action', { id: action.id });
      const done = action.group === 'apps' ? `Opening ${action.label}` : DONE[action.id];
      if (done) Ui.hud(done);
      return true;
    } catch {
      window.haptic('warn');
      return false;
    }
  }

  function setBlocked(blocked) {
    if (blocked && $('blocked').hidden) window.haptic('warn');
    $('blocked').hidden = !blocked;
  }

  // ---- The way in: a code from the QR or typed, then a yes on the PC ----

  const CODE_HINT = 'On your PC, open the island’s Deck tab and choose Add phone. Then type the code here.';
  const UNREACHABLE = 'Can’t reach your PC. Is PC Deck on, and is this phone on the same Wi-Fi?';

  async function pair(code) {
    Splash.say('Connecting…');
    const claim = await post('/api/pair', { code });
    if (claim.status !== 200) {
      if (readToken()) return readToken();   // an old link in the address bar: this phone is already allowed
      Splash.say(claim.error || 'That code did not work. Show a new one on your PC.', 'bad');
      return null;
    }
    history.replaceState(null, '', location.pathname);   // the code is spent: it stays out of the address bar
    Splash.say(`Allow this phone on ${claim.data.pc}`, 'wait');
    for (;;) {
      await wait(PAIR_POLL_MS);
      const poll = await post('/api/pair/poll', { ticket: claim.data.ticket });
      const state = poll.data?.state;
      if (state === 'approved') {
        saveToken(poll.data.token);
        window.haptic('success');
        return poll.data.token;
      }
      if (state !== 'waiting') {
        window.haptic('warn');
        Splash.say(state === 'refused' ? 'Refused on the PC. Show a new code to try again.' : 'Nobody answered on the PC. Show a new code to try again.', 'bad');
        return null;
      }
    }
  }

  async function connect() {
    const match = location.hash.match(/^#p=([\w-]+)$/);
    try {
      return match ? await pair(match[1]) : readToken();
    } catch {
      Splash.say(UNREACHABLE, 'bad');
      return match ? null : readToken();
    }
  }

  // Nothing to scan here, or the last try failed. There is no reloading a Home Screen app, so this always has a way on.
  async function askForCode() {
    if (!$('splash-note').textContent) Splash.say(CODE_HINT);
    for (;;) {
      const code = await Splash.askCode();
      try {
        const allowed = await pair(code);
        if (allowed) return allowed;
      } catch {
        Splash.say(UNREACHABLE, 'bad');
      }
    }
  }

  // ---- Pages ----

  function showPage(id, animate = true) {
    for (const page of PAGES) {
      $(`page-${page.id}`).hidden = page.id !== id;
      $('dock').querySelector(`[data-page="${page.id}"]`)?.setAttribute('aria-current', page.id === id ? 'page' : 'false');
    }
    if (animate) Ui.flash($(`page-${id}`), 'just-shown', 400);
    try { localStorage.setItem(PAGE_KEY, id); } catch { /* the choice just won't be remembered */ }
  }

  function buildDock() {
    const dock = $('dock');
    for (const page of PAGES) {
      const button = document.createElement('button');
      button.type = 'button';
      button.dataset.page = page.id;
      button.append(Ui.icon(page.id));
      const label = document.createElement('span');
      label.textContent = page.label;
      button.append(label);
      button.addEventListener('click', () => {
        window.haptic('tick');
        showPage(page.id);
      });
      dock.append(button);
    }
    let saved = 'now';
    try { saved = localStorage.getItem(PAGE_KEY) ?? 'now'; } catch { /* the first page, then */ }
    showPage(PAGES.some((page) => page.id === saved) ? saved : 'now', false);
  }

  function showMode(id) {
    for (const mode of MODES) {
      $(`mode-${mode.id}`).hidden = mode.id !== id;
      $('mode-chips').querySelector(`[data-mode="${mode.id}"]`)?.setAttribute('aria-selected', String(mode.id === id));
    }
    Ui.flash($(`mode-${id}`), 'just-shown', 400);
    try { localStorage.setItem(MODE_KEY, id); } catch { /* the choice just won't be remembered */ }
  }

  function buildModes() {
    const chips = $('mode-chips');
    for (const mode of MODES) {
      const chip = document.createElement('button');
      chip.type = 'button';
      chip.role = 'tab';
      chip.dataset.mode = mode.id;
      chip.textContent = mode.label;
      chip.addEventListener('click', () => {
        window.haptic('tick');
        showMode(mode.id);
      });
      chips.append(chip);
    }
    let saved = 'game';
    try { saved = localStorage.getItem(MODE_KEY) ?? 'game'; } catch { /* the first mode, then */ }
    showMode(MODES.some((mode) => mode.id === saved) ? saved : 'game');
  }

  // ---- Keys ----

  function monogram(label) {
    return Object.assign(document.createElement('span'), { className: 'monogram', textContent: label.charAt(0) });
  }

  function loadAppIcon(action, image) {
    blob(`/api/icon/${action.id.slice('app:'.length)}`).then((url) => {
      if (url) image.src = url;
      else image.replaceWith(monogram(action.label));
    }).catch(() => image.replaceWith(monogram(action.label)));
  }

  // ---- The Apps page: every tile is an app; the last one adds another, and Edit takes them off ----

  let editingApps = false;

  function renderApps(actions) {
    const host = $('apps');
    host.replaceChildren();
    actions.filter((action) => action.group === 'apps').forEach((action, position) => {
      const tile = Ui.key(action, position % 24, async (run) => (editingApps ? removeApp(run) : runAction(run)), loadAppIcon);
      const badge = document.createElement('span');
      badge.className = 'remove-badge';
      badge.append(Ui.icon('x'));
      tile.append(badge);
      host.append(tile);
    });
    const add = document.createElement('button');
    add.type = 'button';
    add.className = 'key add-tile';
    add.append(Ui.icon('plus'), Object.assign(document.createElement('span'), { className: 'key-label', textContent: 'Add' }));
    add.addEventListener('pointerdown', () => window.haptic('click'));
    add.addEventListener('click', openSheet);
    host.append(add);
  }

  async function reloadApps() {
    renderApps((await api('/api/deck')).actions);
  }

  async function removeApp(action) {
    try {
      await api('/api/apps/remove', { id: action.id });
      window.haptic('confirm');
      Ui.hud(`Removed ${action.label}`);
      await reloadApps();
      return true;
    } catch {
      window.haptic('warn');
      return false;
    }
  }

  function bindAppsEdit() {
    $('apps-edit').addEventListener('click', () => {
      editingApps = !editingApps;
      window.haptic('tick');
      $('apps-edit').textContent = editingApps ? 'Done' : 'Edit';
      $('apps').classList.toggle('editing', editingApps);
    });
  }

  let choices = [];
  let iconWatcher = null;

  // Icons come only as a row scrolls into view: a PC can have hundreds of apps. The first letter stays if one has none.
  function watchIcon(row) {
    iconWatcher ??= new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        iconWatcher.unobserve(entry.target);
        const seen = entry.target;
        blob(`/api/choice-icon/${seen.dataset.choice}`).then((url) => {
          if (!url) return;
          const image = new Image();
          image.alt = '';
          image.src = url;
          seen.querySelector('.monogram')?.replaceWith(image);
        }).catch(() => {});
      }
    }, { root: $('sheet-list'), rootMargin: '200px' });
    iconWatcher.observe(row);
  }

  function renderChoices() {
    const query = $('sheet-search').value.trim().toLowerCase();
    const list = $('sheet-list');
    list.replaceChildren();
    const shown = choices.filter((choice) => choice.name.toLowerCase().includes(query));
    for (const choice of shown) {
      const row = document.createElement('button');
      row.type = 'button';
      row.className = 'choice';
      row.dataset.choice = choice.id.slice('choice:'.length);
      row.append(monogram(choice.name), Object.assign(document.createElement('span'), { className: 'choice-name', textContent: choice.name }), Ui.icon('plus'));
      row.addEventListener('click', () => addChoice(choice, row));
      list.append(row);
      watchIcon(row);
    }
    if (shown.length === 0) list.append(Object.assign(document.createElement('p'), { className: 'empty', textContent: query ? 'No app by that name.' : 'Every app is already on your deck.' }));
  }

  async function addChoice(choice, row) {
    try {
      await api('/api/apps/add', { id: choice.id });
      window.haptic('success');
      Ui.hud(`Added ${choice.name}`);
      choices = choices.filter((other) => other !== choice);
      row.remove();
      await reloadApps();
    } catch (error) {
      window.haptic('warn');
      Ui.hud(error.message);
      Ui.flash(row, 'fail');
    }
  }

  async function openSheet() {
    $('sheet').hidden = false;
    $('sheet-search').value = '';
    $('sheet-list').replaceChildren(Object.assign(document.createElement('p'), { className: 'empty', textContent: 'Reading your PC’s apps…' }));
    try {
      choices = (await api('/api/apps/choices')).choices;
      renderChoices();
    } catch {
      $('sheet-list').replaceChildren(Object.assign(document.createElement('p'), { className: 'empty', textContent: 'Could not read your PC’s apps. Try again.' }));
    }
  }

  function bindSheet() {
    $('sheet-close').addEventListener('click', () => {
      window.haptic('tick');
      $('sheet').hidden = true;
    });
    $('sheet-search').addEventListener('input', renderChoices);
  }

  function renderActions(actions) {
    const places = {
      game: $('group-game'), present: $('group-present'),
      movie: $('group-movie'), work: $('group-work'), sys: $('group-sys'),
    };
    renderApps(actions);
    actions.forEach((action, position) => {
      if (action.id === 'present:next') {
        const big = Ui.key(action, 0, runAction);
        big.classList.add('big');
        $('present-main').append(big);
      } else if (action.id === 'present:start') {
        const key = Ui.key(action, position % 24, async (run) => {
          startPresentClock();
          return runAction(run);
        });
        places.present.append(key);
      } else {
        places[action.group]?.append(Ui.key(action, position % 24, runAction));
      }
    });
    buildTransport(actions);
  }

  let presentStart = 0;
  function startPresentClock() {
    presentStart = Date.now();
  }
  setInterval(() => {
    if (!presentStart) return;
    const seconds = Math.floor((Date.now() - presentStart) / 1000);
    $('present-clock').textContent = `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
  }, 1000);

  function buildTransport(actions) {
    const byId = (id) => actions.find((action) => action.id === id);
    const host = $('transport');
    for (const [id, cls] of [['media:prev', ''], ['media:play-pause', 'main'], ['media:next', '']]) {
      const action = byId(id);
      const button = document.createElement('button');
      button.type = 'button';
      button.className = `round ${cls}`;
      button.id = id === 'media:play-pause' ? 'play-button' : '';
      button.setAttribute('aria-label', action.label);
      button.append(Ui.icon(id === 'media:play-pause' ? 'play' : action.icon));
      button.addEventListener('pointerdown', () => window.haptic('click'));
      button.addEventListener('click', () => runAction(action));
      host.append(button);
    }
    $('vol-mute').append(Ui.icon('volume'));
    $('vol-mute').addEventListener('click', () => {
      window.haptic('click');
      runAction(byId('media:mute'));
    });
  }

  // ---- Sliders: a level goes out at most every few milliseconds, and the phone ticks as it passes each step ----

  function bindLevel(input, send, onChange) {
    let last = 0;
    let pending = false;
    let lastStep = -1;
    const flush = async () => {
      if (pending) return;
      pending = true;
      await wait(Math.max(0, SEND_LEVEL_MS - (Date.now() - last)));
      pending = false;
      last = Date.now();
      send().catch(() => {});
    };
    input.addEventListener('input', () => {
      const step = Math.round((input.value / input.max) * 20);
      if (step !== lastStep) window.haptic('soft');
      lastStep = step;
      onChange();
      flush();
    });
  }

  function clock(seconds) {
    const whole = Math.max(0, Math.floor(seconds));
    return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`;
  }

  function bindSliders() {
    const volume = $('volume');
    const paint = (input) => input.style.setProperty('--fill', `${(input.value / input.max) * 100}%`);
    bindLevel(volume, () => api('/api/volume', { level: volume.value / 100 }), () => {
      $('vol-value').textContent = volume.value;
      paint(volume);
    });
    for (const [input, flag] of [[volume, (on) => { draggingVolume = on; }], [$('seek'), (on) => { draggingSeek = on; }]]) {
      input.addEventListener('pointerdown', () => flag(true));
      for (const type of ['pointerup', 'pointercancel', 'blur']) input.addEventListener(type, () => flag(false));
    }
    const seek = $('seek');
    seek.addEventListener('input', () => paint(seek));
    seek.addEventListener('change', () => {
      window.haptic('confirm');
      api('/api/seek', { position: seek.value / seek.max }).catch(() => {});
    });
    paint(volume);
    paint(seek);
  }

  // ---- What the PC says ----

  async function showArt(media) {
    if (media.artVersion === artVersion) return;
    artVersion = media.artVersion;
    const url = await blob('/api/art').catch(() => null);
    if (artUrl) URL.revokeObjectURL(artUrl);
    artUrl = url;
    $('art').hidden = !url;
    $('art-empty').hidden = !!url;
    if (url) {
      $('art').src = url;
      $('backdrop').style.backgroundImage = `url(${url})`;
    } else {
      $('backdrop').style.backgroundImage = '';
    }
    $('backdrop').classList.toggle('on', !!url);
  }

  function showMedia(media) {
    document.body.classList.toggle('has-media', !!media);
    if (!media) {
      $('title').textContent = 'Nothing playing';
      $('artist').textContent = 'Start some music on your PC';
      $('art').hidden = true;
      $('art-empty').hidden = false;
      $('backdrop').classList.remove('on');
      artVersion = -1;
      return;
    }
    $('title').textContent = media.title || 'Playing';
    $('artist').textContent = media.artist;
    const play = $('play-button');
    if (play && play.dataset.icon !== String(media.playing)) {
      play.dataset.icon = String(media.playing);
      play.replaceChildren(Ui.icon(media.playing ? 'pause' : 'play'));
    }
    const seek = $('seek');
    $('seek-wrap').hidden = !media.canSeek;
    if (!draggingSeek) {
      seek.value = Math.round(media.position * seek.max);
      seek.style.setProperty('--fill', `${media.position * 100}%`);
      $('time-now').textContent = clock(media.position * media.durationSeconds);
      $('time-end').textContent = clock(media.durationSeconds);
    }
    showArt(media);
  }

  function showStatus(status) {
    Ui.updateMeters(meterViews, status);
    setBlocked(status.inputBlocked === true);
    showMedia(status.media);
    if (!draggingVolume) {
      const volume = $('volume');
      volume.value = Math.round(status.volume * 100);
      volume.style.setProperty('--fill', `${volume.value}%`);
      $('vol-value').textContent = status.muted ? 'Mute' : volume.value;
    }
    document.body.classList.toggle('muted', status.muted);
    if (micMuted !== status.micMuted) {
      micMuted = status.micMuted;
      document.querySelector('[data-action="pc:mic"]')?.classList.toggle('lit', micMuted);
    }
  }

  function goOffline() {
    document.body.classList.add('offline');
    Ui.island('PC Deck is off');
    window.haptic('warn');
  }

  function backOnline() {
    document.body.classList.remove('offline');
    Ui.island(pcName);
    Ui.hud('Connected');
    window.haptic('confirm');
  }

  async function pollStatus() {
    if (document.visibilityState === 'visible') {
      try {
        showStatus(await api('/api/status'));
        if (misses >= OFFLINE_AFTER) backOnline();
        misses = 0;
      } catch (error) {
        if (error.status === 401) {
          forgetToken();
          location.replace(location.pathname);
          return;
        }
        misses++;
        if (misses === OFFLINE_AFTER) goOffline();
      }
    }
    setTimeout(pollStatus, POLL_MS);
  }

  // ---- Immersion: full screen and an awake screen, both of which a browser only gives after a tap ----

  async function keepAwake() {
    try { await navigator.wakeLock?.request('screen'); } catch { /* the screen may still sleep */ }
  }

  function immerse() {
    document.documentElement.requestFullscreen?.({ navigationUI: 'hide' }).catch(() => {});
    keepAwake();
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible') keepAwake();
    });
  }

  function applyLook(info) {
    pcName = info.pc || 'PC';
    if (/^#[0-9a-f]{6}$/i.test(info.accent ?? '')) document.documentElement.style.setProperty('--accent', info.accent);
    Ui.island(pcName);
    $('pc-name').textContent = pcName;
  }

  async function bindBlockedEscape() {
    $('blocked-escape').addEventListener('click', async () => {
      window.haptic('click');
      try {
        await api('/api/action', { id: 'pc:desktop' });
        setBlocked(false);
      } catch {
        // Still blocked: the notice stays and the next reading updates it.
      }
    });
  }

  // An iPhone's browsers cannot put a page in full screen, but a Home Screen app has no bars at all, and nobody finds
  // that alone. The hint shows the buttons to look for and stays until it is closed, then never comes back. With no
  // memory to keep that promise, it is better not said.
  function offerHomeScreen() {
    const agent = navigator.userAgent;
    const isApp = navigator.standalone === true || matchMedia('(display-mode: standalone)').matches;
    if (isApp || !/iPhone/.test(agent)) return;
    try {
      if (localStorage.getItem(HOME_TIP_KEY)) return;
    } catch {
      return;
    }
    const isSafari = !/CriOS|FxiOS|EdgiOS|OPiOS/.test(agent);
    // Safari 26's default layout keeps Share behind the ··· at the right of its bar; before that it sat in the middle. The page cannot see which layout is on.
    const shareIsBehindMore = isSafari && Number(agent.match(/Version\/(\d+)/)?.[1]) >= 26;

    const step = (icon, label = '', spoken = label) => {
      const item = document.createElement('span');
      const mark = document.createElement('i');
      mark.setAttribute('aria-label', spoken);
      mark.append(Ui.icon(icon));
      item.append(mark);
      if (label) item.append(label);
      return item;
    };
    const then = () => Object.assign(document.createElement('span'), { className: 'tip-then', textContent: '›' });
    $('home-tip-steps').append(
      ...(shareIsBehindMore ? [step('more', '', 'More'), then()] : []),
      step('share', 'Share'),
      then(),
      step('add-home', 'Add to Home Screen'),
    );

    const tip = $('home-tip');
    if (isSafari) tip.dataset.arrow = shareIsBehindMore ? 'right' : 'center';
    $('home-tip-arrow').append(Ui.icon('down'));
    const close = $('home-tip-close');
    close.append(Ui.icon('x'));
    close.addEventListener('click', () => {
      window.haptic('tick');
      tip.hidden = true;
      try { localStorage.setItem(HOME_TIP_KEY, '1'); } catch { /* it may come back next time */ }
    });
    tip.hidden = false;
  }

  async function start() {
    await Splash.play();
    token = await connect() ?? await askForCode();
    let info;
    while (!info) {
      try {
        info = await api('/api/deck');
      } catch (error) {
        if (error.status !== 401) {
          Splash.say('PC Deck is off. Turn it on in the island’s Deck tab on your PC.', 'wait');
          setTimeout(() => location.reload(), RETRY_MS);
          return;
        }
        forgetToken();
        Splash.say('This phone was removed from the PC. Choose Add phone there and type the new code.', 'bad');
        token = await askForCode();
      }
    }
    applyLook(info);
    buildDock();
    buildModes();
    $('type-toggle').append(Ui.icon('keyboard'));
    meterViews = Ui.meters();
    renderActions(info.actions);
    bindSliders();
    bindBlockedEscape();
    bindAppsEdit();
    bindSheet();
    window.bindTrackpad(api);
    pollStatus();

    await Splash.waitForTap();
    window.haptic('open');
    immerse();
    $('deck').hidden = false;
    $('dock').hidden = false;
    $('island').hidden = false;
    document.body.classList.add('entered');
    await Splash.leave();
    Ui.hud(`Connected to ${pcName}`);
    offerHomeScreen();
  }

  start();
})();
