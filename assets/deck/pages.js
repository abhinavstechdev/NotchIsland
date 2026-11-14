'use strict';

// Drawing the deck: keys, the dock, the gauges and the phone's own island. No talking to the PC here.
window.Ui = (() => {
  const SVG_NS = 'http://www.w3.org/2000/svg';
  const HOLD_MS = 900;
  const HOLD_TICK_MS = 150;
  const HUD_MS = 1500;

  const ICONS = {
    now: ['M9 18V5l11-2v13', 'M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0z', 'M20 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z'],
    pad: ['M4 4h16v16H4z', 'M4 15h16', 'M12 15v5'],
    apps: ['M4 4h6v6H4z', 'M14 4h6v6h-6z', 'M4 14h6v6H4z', 'M14 14h6v6h-6z'],
    modes: ['M4 7h16', 'M4 12h16', 'M4 17h16', 'M9 5v4', 'M15 10v4', 'M7 15v4'],
    pc: ['M3 4h18v12H3z', 'M8 20h8', 'M12 16v4'],
    prev: ['M19 5v14l-9-7z', 'M6 5v14'],
    'play-pause': ['M5 5v14l8-7z', 'M16 5v14', 'M20 5v14'],
    play: ['M7 4v16l13-8z'],
    pause: ['M8 5v14', 'M16 5v14'],
    next: ['M5 5v14l9-7z', 'M18 5v14'],
    mute: ['M4 9v6h4l5 4V5L8 9z', 'M17 9l4 6', 'M21 9l-4 6'],
    volume: ['M4 9v6h4l5 4V5L8 9z', 'M16.5 9a4 4 0 0 1 0 6', 'M19 6.5a8 8 0 0 1 0 11'],
    game: ['M7 8h10a4 4 0 0 1 4 4v3a3 3 0 0 1-5.4 1.8L14.5 15h-5l-1.1 1.8A3 3 0 0 1 3 15v-3a4 4 0 0 1 4-4z', 'M8 11v3', 'M6.5 12.5h3', 'M16 12h.01'],
    camera: ['M4 8h3l2-3h6l2 3h3v11H4z', 'M12 16a3 3 0 1 0 0-6 3 3 0 0 0 0 6z'],
    clip: ['M4 5h16v14H4z', 'M4 9h16', 'M8 5v4', 'M16 5v4', 'M10 12v4l4-2z'],
    record: ['M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16z', 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z'],
    switch: ['M4 8h13', 'M13 4l4 4-4 4', 'M20 16H7', 'M11 12l-4 4 4 4'],
    mic: ['M12 3a3 3 0 0 0-3 3v5a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3z', 'M6 11a6 6 0 0 0 12 0', 'M12 17v4'],
    left: ['M15 5l-7 7 7 7'],
    right: ['M9 5l7 7-7 7'],
    blank: ['M3 5h18v13H3z', 'M3 5l18 13'],
    stop: ['M6 6h12v12H6z'],
    full: ['M4 9V4h5', 'M15 4h5v5', 'M20 15v5h-5', 'M9 20H4v-5'],
    snip: ['M4 9V4h5', 'M15 4h5v5', 'M20 15v5h-5', 'M9 20H4v-5', 'M9 12h6', 'M12 9v6'],
    desktop: ['M3 4h18v12H3z', 'M8 20h8', 'M12 16v4'],
    tasks: ['M3 6h8v12H3z', 'M14 6h7v5h-7z', 'M14 14h7v4h-7z'],
    copy: ['M9 9h11v11H9z', 'M5 15H4V4h11v1'],
    paste: ['M8 4h8v3H8z', 'M8 5H5v15h14V5h-3'],
    undo: ['M9 14L4 9l5-5', 'M4 9h10a6 6 0 0 1 0 12h-3'],
    close: ['M3 5h18v14H3z', 'M9 9l6 6', 'M15 9l-6 6'],
    lock: ['M6 11h12v9H6z', 'M8.5 11V7.5a3.5 3.5 0 0 1 7 0V11'],
    sleep: ['M20 14.5A8.5 8.5 0 1 1 9.5 4a7 7 0 0 0 10.5 10.5z'],
    restart: ['M4 12a8 8 0 1 0 2.6-5.9', 'M4 4v4.5h4.5'],
    power: ['M12 3v9', 'M7 6.3a8 8 0 1 0 10 0'],
    keyboard: ['M3 6h18v12H3z', 'M7 10h.01', 'M11 10h.01', 'M15 10h.01', 'M18 10h.01', 'M7 14h10'],
    plus: ['M12 5v14', 'M5 12h14'],
    x: ['M6 6l12 12', 'M18 6L6 18'],
    erase: ['M9 5h12v14H9l-6-7z', 'M12 9.5l5 5', 'M17 9.5l-5 5'],
    more: ['M5 12h.01', 'M12 12h.01', 'M19 12h.01'],
    share: ['M12 15V3', 'M8 7l4-4 4 4', 'M8 11H5v10h14V11h-3'],
    'add-home': ['M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z', 'M12 8v8', 'M8 12h8'],
    down: ['M12 4v16', 'M6 14l6 6 6-6'],
  };

  const METERS = [
    { key: 'cpu', label: 'CPU', unit: '%', warm: 70, hot: 90, max: 100 },
    { key: 'gpu', label: 'GPU', unit: '%', warm: 80, hot: 95, max: 100 },
    { key: 'ram', label: 'RAM', unit: '%', warm: 75, hot: 90, max: 100 },
    { key: 'temp', label: 'Temp', unit: '°', warm: 72, hot: 85, max: 100 },
  ];

  function icon(name) {
    const svg = document.createElementNS(SVG_NS, 'svg');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('aria-hidden', 'true');
    for (const d of ICONS[name] || []) {
      const path = document.createElementNS(SVG_NS, 'path');
      path.setAttribute('d', d);
      svg.append(path);
    }
    return svg;
  }

  function flash(element, className, ms = 520) {
    element.classList.add(className);
    setTimeout(() => element.classList.remove(className), ms);
  }

  // Press and hold for what can't be undone: the ring fills and the phone ticks faster as it nears the end.
  function bindHold(key, label, run) {
    let timer = null;
    let ticker = null;
    const cancel = () => {
      clearTimeout(timer);
      clearTimeout(ticker);
      timer = ticker = null;
      key.classList.remove('pressing');
    };
    const tick = (gap) => {
      window.haptic('tick');
      ticker = setTimeout(() => tick(Math.max(50, gap - 30)), gap);
    };
    key.style.setProperty('--hold-ms', `${HOLD_MS}ms`);
    key.addEventListener('pointerdown', () => {
      key.classList.add('pressing');
      tick(HOLD_TICK_MS);
      timer = setTimeout(() => {
        cancel();
        window.haptic('success');
        run();
      }, HOLD_MS);
    });
    key.addEventListener('pointerup', () => {
      const early = timer !== null;
      cancel();
      if (!early) return;
      const original = label.textContent;
      label.textContent = 'Hold';
      flash(key, 'hint', 1100);
      setTimeout(() => { label.textContent = original; }, 1100);
    });
    for (const type of ['pointerleave', 'pointercancel']) key.addEventListener(type, cancel);
    key.addEventListener('contextmenu', (event) => event.preventDefault());
  }

  function key(action, position, run, loadIcon) {
    const element = document.createElement('button');
    element.type = 'button';
    element.className = 'key';
    element.dataset.action = action.id;
    element.style.setProperty('--i', position);
    element.setAttribute('aria-label', action.confirm ? `${action.label}, press and hold` : action.label);
    if (action.group === 'apps') {
      const image = document.createElement('img');
      image.className = 'key-icon';
      image.alt = '';
      element.append(image);
      loadIcon?.(action, image, element);
    } else {
      element.append(icon(action.icon));
    }
    const label = document.createElement('span');
    label.className = 'key-label';
    label.textContent = action.label;
    element.append(label);

    const go = async () => {
      flash(element, await run(action) ? 'ok' : 'fail');
    };
    if (action.confirm) {
      element.classList.add('hold');
      bindHold(element, label, go);
    } else {
      element.addEventListener('pointerdown', () => window.haptic('click'));
      element.addEventListener('click', go);
    }
    return element;
  }

  function meters() {
    const views = [];
    for (const host of document.querySelectorAll('[data-meters]')) {
      for (const meter of METERS) {
        const box = document.createElement('div');
        box.className = 'meter';
        box.style.setProperty('--level', 0);
        const value = document.createElement('b');
        value.textContent = '--';
        const label = document.createElement('span');
        label.textContent = meter.label;
        box.append(value, label);
        host.append(box);
        views.push({ box, value, meter });
      }
    }
    return views;
  }

  function updateMeters(views, status) {
    for (const { box, value, meter } of views) {
      const reading = status[meter.key];
      const known = typeof reading === 'number';
      value.textContent = known ? `${reading}${meter.unit}` : '--';
      box.style.setProperty('--level', known ? Math.min(1, reading / meter.max) : 0);
      box.classList.toggle('warm', known && reading >= meter.warm && reading < meter.hot);
      box.classList.toggle('hot', known && reading >= meter.hot);
    }
  }

  // The phone's own island: at rest it names the PC; anything that happens grows it for a moment, as on the PC.
  const islandEl = document.getElementById('island');
  const islandText = document.getElementById('island-text');
  let resting = '';
  let hudTimer = null;

  function island(text) {
    resting = text;
    if (!hudTimer) islandText.textContent = text;
  }

  function hud(text) {
    clearTimeout(hudTimer);
    islandText.textContent = text;
    islandEl.classList.add('open');
    hudTimer = setTimeout(() => {
      hudTimer = null;
      islandEl.classList.remove('open');
      islandText.textContent = resting;
    }, HUD_MS);
  }

  return { icon, key, meters, updateMeters, island, hud, flash };
})();
