'use strict';

window.bindTrackpad = (api) => {
  const SENSITIVITY = 2.2;
  const SCROLL_STEP_PX = 28;
  const TAP_MS = 220;
  const TAP_SLOP_PX = 8;
  // How long a tap waits to see if a second touch turns it into a drag or double-click.
  const SECOND_TOUCH_MS = 200;
  const MAX_QUEUE = 64;
  const RING_MS = 450;

  const pad = document.getElementById('pad');
  const glow = document.getElementById('pad-glow');
  const typeForm = document.getElementById('type-form');
  const typeToggle = document.getElementById('type-toggle');
  const pointers = new Map();
  const carry = { dx: 0, dy: 0 };
  const queue = [];
  let gesture = null;
  let scrollCarry = 0;
  let sending = false;
  let pendingClick = null;
  let dragging = false;
  let padBox = pad.getBoundingClientRect();

  function moveGlow(event) {
    glow.style.transform = `translate(${event.clientX - padBox.left}px, ${event.clientY - padBox.top}px)`;
  }

  function showTapRing(event) {
    const ring = document.createElement('div');
    ring.className = 'pad-ring';
    ring.style.setProperty('--x', `${event.clientX - padBox.left}px`);
    ring.style.setProperty('--y', `${event.clientY - padBox.top}px`);
    pad.append(ring);
    setTimeout(() => ring.remove(), RING_MS);
  }

  // Everything goes out one at a time and in order, so a drag can't drop before its last move.
  async function pump() {
    if (sending) return;
    sending = true;
    while (queue.length > 0) {
      try {
        await api('/api/mouse', queue.shift());
      } catch {
        // Offline: later input still gets its turn, and the PC lets go of a stuck button itself.
      }
    }
    sending = false;
  }

  function enqueue(command) {
    const last = queue[queue.length - 1];
    if (command.op === 'move' && last?.op === 'move') {
      queue[queue.length - 1] = { ...last, dx: last.dx + command.dx, dy: last.dy + command.dy };
    } else if (command.op === 'scroll' && last?.op === 'scroll') {
      queue[queue.length - 1] = { ...last, amount: last.amount + command.amount };
    } else if (queue.length < MAX_QUEUE) {
      queue.push(command);
    }
    pump();
  }

  function enqueueMove(deltaX, deltaY) {
    carry.dx += deltaX * SENSITIVITY;
    carry.dy += deltaY * SENSITIVITY;
    const dx = Math.trunc(carry.dx);
    const dy = Math.trunc(carry.dy);
    if (dx === 0 && dy === 0) return;
    carry.dx -= dx;
    carry.dy -= dy;
    enqueue({ op: 'move', dx, dy });
  }

  const click = (button) => enqueue({ op: 'click', button });

  function startDrag() {
    dragging = true;
    pad.classList.add('dragging');
    window.haptic('confirm');
    enqueue({ op: 'press', button: 'left' });
  }

  function endDrag() {
    if (!dragging) return;
    dragging = false;
    pad.classList.remove('dragging');
    window.haptic('click');
    enqueue({ op: 'release', button: 'left' });
  }

  function centerY() {
    const points = [...pointers.values()];
    return points.reduce((sum, point) => sum + point.y, 0) / points.length;
  }

  function beginGesture(event) {
    // A touch soon after a tap might become a drag, so that tap's click stays on hold until we know.
    const afterTap = pendingClick !== null;
    clearTimeout(pendingClick);
    pendingClick = null;
    gesture = { start: event.timeStamp, moved: 0, fingers: 1, afterTap };
  }

  pad.addEventListener('pointerdown', (event) => {
    event.preventDefault();
    pad.setPointerCapture(event.pointerId);
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    padBox = pad.getBoundingClientRect();
    moveGlow(event);
    pad.classList.add('active', 'used');
    if (pointers.size === 1) {
      beginGesture(event);
    } else if (gesture) {
      gesture.fingers = Math.max(gesture.fingers, pointers.size);
    }
    if (gesture) gesture.lastCenterY = centerY();
  });

  function scrollWithTwoFingers() {
    const center = centerY();
    scrollCarry += gesture.lastCenterY - center;
    gesture.lastCenterY = center;
    const steps = Math.trunc(scrollCarry / SCROLL_STEP_PX);
    if (steps !== 0) {
      scrollCarry -= steps * SCROLL_STEP_PX;
      window.haptic('soft');
      enqueue({ op: 'scroll', amount: -steps });
    }
  }

  pad.addEventListener('pointermove', (event) => {
    const point = pointers.get(event.pointerId);
    if (!point || !gesture) return;
    const deltaX = event.clientX - point.x;
    const deltaY = event.clientY - point.y;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    moveGlow(event);
    gesture.moved += Math.abs(deltaX) + Math.abs(deltaY);

    if (pointers.size >= 2) {
      scrollWithTwoFingers();
      return;
    }
    if (gesture.afterTap && !dragging && gesture.moved >= TAP_SLOP_PX) startDrag();
    enqueueMove(deltaX, deltaY);
  });

  function finishGesture(finished, event) {
    const isTap = event.timeStamp - finished.start < TAP_MS && finished.moved < TAP_SLOP_PX;
    if (isTap) {
      window.haptic('click');
      showTapRing(event);
    }
    if (finished.afterTap) {
      // The earlier tap's click, plus this one if it was a tap too: a double-click.
      click('left');
      if (isTap) click('left');
    } else if (isTap && finished.fingers > 1) {
      click('right');
    } else if (isTap) {
      pendingClick = setTimeout(() => {
        pendingClick = null;
        click('left');
      }, SECOND_TOUCH_MS);
    }
  }

  function endPointer(event) {
    if (!pointers.delete(event.pointerId)) return;
    if (pointers.size > 0) return;
    pad.classList.remove('active');
    const finished = gesture;
    gesture = null;
    scrollCarry = 0;
    if (dragging) {
      endDrag();
    } else if (finished && event.type === 'pointerup') {
      finishGesture(finished, event);
    }
  }

  pad.addEventListener('pointerup', endPointer);
  pad.addEventListener('pointercancel', endPointer);
  pad.addEventListener('contextmenu', (event) => event.preventDefault());

  // Left and Right are real buttons: hold one and move on the pad with another finger to drag.
  const heldButtons = new Map();

  function bindMouseButton(id, button) {
    const element = document.getElementById(id);
    const letGo = (event) => {
      if (!heldButtons.has(event.pointerId)) return;
      heldButtons.delete(event.pointerId);
      element.classList.remove('held');
      window.haptic('tick');
      enqueue({ op: 'release', button });
    };
    element.addEventListener('pointerdown', (event) => {
      event.preventDefault();
      element.setPointerCapture(event.pointerId);
      heldButtons.set(event.pointerId, letGo);
      element.classList.add('held');
      window.haptic('click');
      enqueue({ op: 'press', button });
    });
    element.addEventListener('pointerup', letGo);
    element.addEventListener('pointercancel', letGo);
    element.addEventListener('contextmenu', (event) => event.preventDefault());
    // A keyboard or switch "click" has no pointer behind it.
    element.addEventListener('click', (event) => {
      if (event.detail === 0) click(button);
    });
  }

  bindMouseButton('pad-left', 'left');
  bindMouseButton('pad-right', 'right');

  // Leaving the page mid-drag would never deliver a pointerup.
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState !== 'hidden') return;
    endDrag();
    for (const [pointerId, letGo] of [...heldButtons]) letGo({ pointerId });
  });

  const input = document.getElementById('type-input');
  typeToggle.addEventListener('click', () => {
    typeForm.hidden = !typeForm.hidden;
    typeToggle.setAttribute('aria-expanded', String(!typeForm.hidden));
    window.haptic('tick');
    if (!typeForm.hidden) input.focus();
  });
  typeForm.addEventListener('submit', async (event) => {
    event.preventDefault();
    const text = input.value;
    if (!text) return;
    try {
      await api('/api/type', { text });
      window.haptic('confirm');
      input.value = '';
    } catch {
      window.haptic('warn');
      input.select();
    }
  });
};
