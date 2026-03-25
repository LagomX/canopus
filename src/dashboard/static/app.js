'use strict';

const API      = '/api/journal';
const timeline = document.getElementById('timeline');
const loading  = document.getElementById('loading');
const empty    = document.getElementById('empty-state');
const input    = document.getElementById('compose-input');
const btn      = document.getElementById('compose-btn');
const errorMsg = document.getElementById('error-msg');

let entries = [];
let errorTimer = null;

// ── Header date ───────────────────────────────────────────────────────────────

(function setHeaderDate() {
  const d = new Date();
  const opts = { weekday: 'long', month: 'long', day: 'numeric' };
  document.getElementById('today-date').textContent =
    d.toLocaleDateString('en-US', opts);
})();

// ── Compose box ───────────────────────────────────────────────────────────────

input.addEventListener('input', function () {
  this.style.height = 'auto';
  this.style.height = Math.min(this.scrollHeight, 400) + 'px';
  btn.disabled = this.value.trim() === '';
});

input.addEventListener('keydown', function (e) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
    e.preventDefault();
    if (!btn.disabled) submitEntry();
  }
});

btn.addEventListener('click', submitEntry);

// ── Compose meta (mood / energy / tags / intentions) ──────────────────────────

let selectedMood = '';

document.querySelectorAll('.mood-btn').forEach(btn => {
  btn.addEventListener('click', function () {
    const v = this.dataset.v;
    selectedMood = (selectedMood === v) ? '' : v;
    document.querySelectorAll('.mood-btn').forEach(b =>
      b.classList.toggle('selected', b.dataset.v === selectedMood)
    );
  });
});

function resetComposeMeta() {
  selectedMood = '';
  document.querySelectorAll('.mood-btn').forEach(b => b.classList.remove('selected'));
  document.getElementById('compose-tags').value = '';
}

// ── Load & render ─────────────────────────────────────────────────────────────

async function loadEntries() {
  try {
    const res = await fetch(API + '?days=7');
    if (!res.ok) throw new Error('Server error ' + res.status);
    entries = await res.json();
    renderTimeline();
  } catch (err) {
    showError('加载失败，请刷新页面');
  } finally {
    loading.style.display = 'none';
    timeline.classList.add('loaded');
  }
}

function renderTimeline() {
  document.querySelectorAll('.date-group').forEach(el => el.remove());
  empty.style.display = 'none';

  if (entries.length === 0) {
    empty.style.display = 'block';
    return;
  }

  const todayS     = todayStr();
  const yesterdayS = yesterdayStr();
  const groups     = groupByDate(entries);

  for (const [date, dayEntries] of Object.entries(groups)) {
    const group = document.createElement('div');
    group.className = 'date-group';
    group.dataset.date = date;

    const hdr = document.createElement('div');
    hdr.className = 'date-header';
    hdr.textContent = dateLabel(date, todayS, yesterdayS);
    group.appendChild(hdr);

    const divider = document.createElement('div');
    divider.className = 'date-divider';
    group.appendChild(divider);

    dayEntries.forEach(entry => group.appendChild(makeEntryEl(entry)));
    timeline.appendChild(group);
  }
}

function groupByDate(list) {
  const groups = {};
  for (const entry of list) {
    if (!groups[entry.date]) groups[entry.date] = [];
    groups[entry.date].push(entry);
  }
  return groups;
}

function makeEntryEl(entry) {
  const card = document.createElement('div');
  card.className = 'entry-card';
  card.dataset.id = entry.id;

  const time = document.createElement('div');
  time.className = 'entry-time';
  time.textContent = timeStr(entry.timestamp) + (entry.mood ? '  ' + entry.mood : '');

  const content = document.createElement('div');
  content.className = 'entry-content';
  content.textContent = entry.content;

  const del = document.createElement('button');
  del.className = 'entry-delete';
  del.textContent = '删除';
  del.addEventListener('click', () => deleteEntry(entry.id, card));

  card.appendChild(time);
  card.appendChild(content);
  card.appendChild(del);
  return card;
}

// ── Submit ────────────────────────────────────────────────────────────────────

async function submitEntry() {
  const content = input.value.trim();
  if (!content) return;

  btn.disabled = true;

  try {
    const tagsRaw = document.getElementById('compose-tags').value.trim();
    const payload = {
      content,
      mood: selectedMood || undefined,
      tags: tagsRaw ? tagsRaw.split(/\s+/) : undefined,
    };

    const res = await fetch(API, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });

    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(body.error || '提交失败');
    }

    const entry = await res.json();
    input.value = '';
    input.style.height = '';
    btn.disabled = true;
    resetComposeMeta();

    entries.unshift(entry);
    renderTimeline();
  } catch (err) {
    showError(err.message || '提交失败');
    btn.disabled = input.value.trim() === '';
  }
}

// ── Delete ────────────────────────────────────────────────────────────────────

async function deleteEntry(id, cardEl) {
  if (!confirm('删除这条记录？')) return;

  try {
    const res = await fetch(API + '/' + encodeURIComponent(id), {
      method: 'DELETE',
    });
    if (!res.ok) {
      showError('删除失败，请重试');
      return;
    }

    entries = entries.filter(e => e.id !== id);

    const group = cardEl.closest('.date-group');
    cardEl.remove();

    if (group && group.querySelectorAll('.entry-card').length === 0) {
      group.remove();
    }

    if (entries.length === 0) {
      empty.style.display = 'block';
    }
  } catch (err) {
    showError(err.message || '删除失败');
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function showError(msg) {
  errorMsg.textContent = msg;
  clearTimeout(errorTimer);
  errorTimer = setTimeout(() => { errorMsg.textContent = ''; }, 3000);
}

function getLocalDateStr(date) {
  date = date || new Date();
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

function todayStr() {
  return getLocalDateStr();
}

function yesterdayStr() {
  const d = new Date();
  d.setDate(d.getDate() - 1);
  return getLocalDateStr(d);
}

function dateLabel(dateStr, today, yesterday) {
  if (dateStr === today)     return 'Today';
  if (dateStr === yesterday) return 'Yesterday';
  const [y, mo, d] = dateStr.split('-').map(Number);
  return new Date(y, mo - 1, d).toLocaleDateString('en-US', { month: 'long', day: 'numeric' });
}

function timeStr(timestamp) {
  const parts = (timestamp || '').split('T');
  return parts.length >= 2 ? parts[1].slice(0, 5) : '';
}

// =============================================================================
// SLEEP
// =============================================================================

const SLEEP_API   = '/api/sleep';
const sleepPopover = document.getElementById('sleep-popover');
let sleepData      = [];
let popoverTimer   = null;

// Set date input default to today, max = today
(function initSleepDate() {
  const dateEl = document.getElementById('sleep-date');
  const today  = getLocalDateStr();
  dateEl.value = today;
  dateEl.max   = today;
})();

document.getElementById('sleep-btn').addEventListener('click', submitSleep);

// ── Quality score selector ─────────────────────────────────────────────────────

let selectedQuality = 0;

document.querySelectorAll('.star-btn').forEach(btn => {
  btn.addEventListener('click', function () {
    const v = parseInt(this.dataset.v, 10);
    selectedQuality = (selectedQuality === v) ? 0 : v;  // toggle off if same
    document.querySelectorAll('.star-btn').forEach(b => {
      b.classList.toggle('selected', parseInt(b.dataset.v, 10) <= selectedQuality);
    });
  });
});

// ── Load & render ─────────────────────────────────────────────────────────────

async function loadSleep() {
  try {
    const res = await fetch(SLEEP_API + '?days=7');
    if (!res.ok) throw new Error('sleep load failed');
    sleepData = await res.json();
    renderSleepChart(sleepData);
  } catch (err) {
    console.error('Sleep load error:', err);
  }
}

function renderSleepChart(data) {
  const barsArea = document.getElementById('bars-area');
  barsArea.querySelectorAll('.bar-col').forEach(el => el.remove());

  // Average (skip nulls)
  const valid = data.filter(d => d !== null);
  const avgEl = document.getElementById('sleep-avg');
  if (valid.length > 0) {
    const avg = valid.reduce((s, d) => s + d.duration_hours, 0) / valid.length;
    avgEl.textContent = fmtHrMin(avg) + ' avg';
  } else {
    avgEl.textContent = '— hr — min';
  }

  // Compute Y-axis range from actual times
  // Bedtimes after midnight (< 12h) are normalized to 24+h so they plot
  // below typical evening bedtimes on the continuous time axis.
  const bedHours  = valid.map(d => normBedtime(toHours(d.bedtime))).filter(h => h !== null);
  const wakeHours = valid.map(d => toHours(d.wake_time)).filter(h => h !== null);

  let y_max, y_min;
  if (bedHours.length > 0 && wakeHours.length > 0) {
    y_max = Math.max(...bedHours) + 1;
    y_min = Math.min(...wakeHours) - 1;
  } else {
    y_max = 25; // 01:00 default top
    y_min = 6;  // 06:00 default bottom
  }
  const range = y_max - y_min;

  updateYAxis(y_min, y_max);

  // Build date → entry map from API response
  const dataMap = {};
  data.forEach(entry => { if (entry) dataMap[entry.date] = entry; });

  // Current week: Sunday to Saturday
  const now        = new Date();
  const sunday     = new Date(now);
  sunday.setDate(now.getDate() - now.getDay());
  const weekDates  = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(sunday);
    d.setDate(sunday.getDate() + i);
    return getLocalDateStr(d);
  });
  const todayStr2  = getLocalDateStr();

  weekDates.forEach(dateStr => {
    const entry = dataMap[dateStr] || null;

    const col = document.createElement('div');
    col.className = 'bar-col';

    const wrap = document.createElement('div');
    wrap.className = 'bar-wrap';

    if (entry && entry.bedtime && entry.wake_time) {
      const bedH  = normBedtime(toHours(entry.bedtime));
      const wakeH = toHours(entry.wake_time);

      // top % = distance from y_max down to bedtime
      const topPct    = (y_max - bedH)  / range * 100;
      const heightPct = (bedH  - wakeH) / range * 100;

      const bar = document.createElement('div');
      bar.className = 'bar';
      bar.style.top    = topPct + '%';
      bar.style.height = '0%';

      const popText = fmtHrMin(entry.duration_hours) +
        '\n' + entry.bedtime + ' → ' + entry.wake_time;
      bar.addEventListener('click', e => toggleSleepPopover(e, bar, popText));

      wrap.appendChild(bar);

      // Animate height after first paint
      requestAnimationFrame(() => requestAnimationFrame(() => {
        bar.style.transition = 'height 0.4s ease-out';
        bar.style.height = heightPct + '%';
      }));
    }

    // MM/DD label
    const [, mo, d] = dateStr.split('-');
    const isToday = dateStr === todayStr2;

    const label = document.createElement('div');
    label.className = 'bar-label' + (isToday ? ' bar-label-today' : '');
    label.textContent = mo + '/' + d;

    col.appendChild(wrap);
    col.appendChild(label);
    barsArea.appendChild(col);
  });

  // Today hint
  const hintEl = document.getElementById('sleep-hint');
  hintEl.textContent = dataMap[todayStr2] ? '' : '今晚记得记录睡眠';
}

// ── Time-axis helpers ─────────────────────────────────────────────────────────

function toHours(timeStr) {
  if (!timeStr) return null;
  const [h, m] = timeStr.split(':').map(Number);
  return h + m / 60;
}

// Bedtimes before noon are treated as next-day (e.g. 00:30 → 24.5)
// so they sit below typical evening bedtimes on a continuous axis.
function normBedtime(h) {
  if (h === null) return null;
  return h < 12 ? h + 24 : h;
}

function updateYAxis(y_min, y_max) {
  const yAxis        = document.getElementById('y-axis');
  const gridContainer = document.getElementById('grid-container');
  yAxis.innerHTML        = '';
  gridContainer.innerHTML = '';
  const range = y_max - y_min;
  const step  = 2;
  const start = Math.ceil(y_min / step) * step;
  for (let h = start; h <= y_max; h += step) {
    const pct = (y_max - h) / range * 100;

    const lbl = document.createElement('span');
    lbl.className = 'y-label';
    lbl.style.top = pct + '%';
    lbl.textContent = fmtClock(h);
    yAxis.appendChild(lbl);

    const line = document.createElement('div');
    line.className = 'grid-line';
    line.style.top = pct + '%';
    gridContainer.appendChild(line);
  }
}

function fmtClock(hours) {
  const totalMins = Math.round(hours * 60);
  const h = Math.floor(totalMins / 60) % 24;
  const m = totalMins % 60;
  return String(h).padStart(2, '0') + ':' + String(m).padStart(2, '0');
}

// ── Submit ────────────────────────────────────────────────────────────────────

async function submitSleep() {
  const bedtime  = document.getElementById('sleep-bedtime').value;
  const waketime = document.getElementById('sleep-waketime').value;
  const date     = document.getElementById('sleep-date').value;

  if (!bedtime || !waketime) {
    showError('请填写入睡和起床时间');
    return;
  }

  const sleepBtn = document.getElementById('sleep-btn');
  sleepBtn.disabled = true;

  try {
    const res = await fetch(SLEEP_API, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        bedtime,
        wake_time: waketime,
        date: date || undefined,
        quality_score: selectedQuality || undefined,
      }),
    });

    if (res.status === 400) {
      const err = await res.json().catch(() => ({}));
      showError(err.error || '时间格式无效');
      return;
    }
    if (!res.ok) {
      const body = await res.json().catch(() => ({}));
      throw new Error(body.error || '提交失败');
    }

    const savedEl = document.getElementById('sleep-saved');
    savedEl.textContent = '已记录 ✓';
    setTimeout(() => { savedEl.textContent = ''; }, 2000);

    selectedQuality = 0;
    document.querySelectorAll('.star-btn').forEach(b => b.classList.remove('selected'));

    await loadSleep();
  } catch (err) {
    showError(err.message || '记录失败');
  } finally {
    sleepBtn.disabled = false;
  }
}

// ── Popover ───────────────────────────────────────────────────────────────────

let _activeBar = null;

function toggleSleepPopover(e, bar, text) {
  e.stopPropagation();
  if (_activeBar === bar && sleepPopover.style.display !== 'none') {
    sleepPopover.style.display = 'none';
    _activeBar = null;
    return;
  }
  _activeBar = bar;
  sleepPopover.textContent = text;
  sleepPopover.style.display = 'block';

  const rect = bar.getBoundingClientRect();
  const pw   = sleepPopover.offsetWidth  || 130;
  const ph   = sleepPopover.offsetHeight || 52;

  // Prefer right side, fall back to left if it overflows
  let left = rect.right + 10;
  if (left + pw > window.innerWidth - 8) left = rect.left - pw - 10;
  left = Math.max(8, left);

  // Vertically centered on the bar
  let top = rect.top + rect.height / 2 - ph / 2;
  top = Math.max(8, Math.min(top, window.innerHeight - ph - 8));

  sleepPopover.style.left = left + 'px';
  sleepPopover.style.top  = top  + 'px';
}

document.addEventListener('click', () => {
  sleepPopover.style.display = 'none';
  _activeBar = null;
});

// ── Sleep helpers ─────────────────────────────────────────────────────────────

function fmtHrMin(hours) {
  const h = Math.floor(hours);
  const m = Math.round((hours - h) * 60);
  return h + ' hr ' + m + ' min';
}

// ── Init ──────────────────────────────────────────────────────────────────────

loadEntries();
loadSleep();
