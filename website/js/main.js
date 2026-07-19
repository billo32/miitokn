// Animated app mock: cycles through 5 scenes (access → backups → password →
// progress → done), one frame every 50 ms, 190 frames per loop.
(function () {
  var statusEl = document.getElementById('status');
  var statusDot = document.getElementById('status-dot');
  var statusText = document.getElementById('status-text');
  var scenes = Array.prototype.slice.call(document.querySelectorAll('.scene'));
  var pageDots = Array.prototype.slice.call(document.querySelectorAll('.mock-dots span'));
  var grantBtn = document.getElementById('grant-btn');
  var backupRows = Array.prototype.slice.call(document.querySelectorAll('#scene-list .backup-row'));
  var backupChecks = Array.prototype.slice.call(document.querySelectorAll('#scene-list .backup-check'));
  var pwField = document.getElementById('pw-field');
  var ringArc = document.getElementById('ring-arc');
  var percentEl = document.getElementById('percent');
  var stepEl = document.getElementById('step-label');
  var copyLabel = document.getElementById('copy1');

  var CIRCUMFERENCE = 314.2;
  var STATUS = [
    { text: 'No access', color: '#ff6b6b', dot: '#ff5f57' },
    { text: 'Access granted', color: '#3ddc84', dot: '#28c76f' },
    { text: 'Encrypted', color: '#e0b64a', dot: '#febc2e' },
    { text: 'Working…', color: '#e0b64a', dot: '#febc2e' },
    { text: 'Done', color: '#3ddc84', dot: '#28c76f' },
  ];
  var STEPS = [
    { at: 0, label: 'Copying backup…' },
    { at: 35, label: 'Decrypting database…' },
    { at: 65, label: 'Scanning Mi Home…' },
    { at: 90, label: 'Extracting tokens…' },
  ];

  var f = -1;

  function tick() {
    f = (f + 1) % 190;

    var scene, since;
    if (f < 28) { scene = 0; since = f; }
    else if (f < 64) { scene = 1; since = f - 28; }
    else if (f < 100) { scene = 2; since = f - 64; }
    else if (f < 142) { scene = 3; since = f - 100; }
    else { scene = 4; since = f - 142; }

    scenes.forEach(function (el, i) {
      el.style.display = i === scene ? '' : 'none';
      if (i === scene) el.style.opacity = Math.min(1, since / 4).toFixed(2);
    });

    var st = STATUS[scene];
    statusEl.style.color = st.color;
    statusDot.style.background = st.dot;
    statusText.textContent = st.text;

    pageDots.forEach(function (el, i) {
      var active = i === scene;
      el.style.width = active ? '8px' : '6px';
      el.style.height = active ? '8px' : '6px';
      el.style.background = active ? '#ff5a1f' : '#4a4a4e';
    });

    if (scene === 0) {
      var glow = (2 + 6 * (0.5 + 0.5 * Math.sin(f / 3))).toFixed(1);
      grantBtn.style.boxShadow = '0 0 0 ' + glow + 'px rgba(255,90,31,0.22)';
    } else if (scene === 1) {
      var selId = since > 12 ? 1 : -1;
      backupRows.forEach(function (row, i) {
        row.classList.toggle('selected', i === selId);
        backupChecks[i].textContent = i === selId ? '✓' : '';
      });
    } else if (scene === 2) {
      var dotCount = Math.min(9, Math.max(0, since - 6));
      pwField.textContent = '•'.repeat(dotCount);
      pwField.style.borderColor = dotCount > 0 ? '#ff5a1f' : 'rgba(255,255,255,0.12)';
    } else if (scene === 3) {
      var percent = Math.min(100, Math.round((since / 38) * 100));
      percentEl.textContent = percent + '%';
      ringArc.setAttribute('stroke-dashoffset', (CIRCUMFERENCE * (1 - percent / 100)).toFixed(1));
      var label = STEPS[0].label;
      STEPS.forEach(function (s) { if (percent >= s.at) label = s.label; });
      stepEl.textContent = label;
    } else if (scene === 4) {
      var copied = since > 18 && since < 32;
      copyLabel.textContent = copied ? 'Copied' : 'Copy';
      copyLabel.style.color = copied ? '#3ddc84' : '#ff7a45';
    }
  }

  tick();
  setInterval(tick, 50);
})();
