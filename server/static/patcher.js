// GG-Retro Binary Patcher
// Replaces all GG server hostnames with custom endpoint

// All known hostnames that need to be replaced
const HOSTNAMES = [
  'appmsg.gadu-gadu.pl',    // 19 chars - hub server
  'register.gadu-gadu.pl',  // 21 chars - registration (LONGEST)
  'adserver.gadu-gadu.pl',  // 21 chars - ads
  'update.gadu-gadu.pl',    // 19 chars - auto-update
  'www.gadu-gadu.pl',       // 16 chars - website
  'pubdir.gadu-gadu.pl',    // 19 chars - public directory
  'retr.gadu-gadu.pl',      // 17 chars - retrieval
  'smsat.gadu-gadu.pl',     // 18 chars - SMS
];

const MAX_DOMAIN_LENGTH = Math.max(...HOSTNAMES.map(h => h.length)); // 21 chars

// Convert string to UTF-16LE bytes
function toUtf16Le(str) {
  const buf = new Uint8Array(str.length * 2);
  for (let i = 0; i < str.length; i++) {
    const code = str.charCodeAt(i);
    buf[i * 2] = code & 0xFF;
    buf[i * 2 + 1] = (code >> 8) & 0xFF;
  }
  return buf;
}

// Convert string to ASCII bytes
function toAscii(str) {
  const buf = new Uint8Array(str.length);
  for (let i = 0; i < str.length; i++) {
    buf[i] = str.charCodeAt(i);
  }
  return buf;
}

// Find all occurrences of pattern in data
function findAll(data, pattern) {
  const positions = [];
  for (let i = 0; i <= data.length - pattern.length; i++) {
    let match = true;
    for (let j = 0; j < pattern.length; j++) {
      if (data[i + j] !== pattern[j]) {
        match = false;
        break;
      }
    }
    if (match) positions.push(i);
  }
  return positions;
}

// Replace all occurrences (null-padded to maintain size)
function replaceAll(data, pattern, replacement) {
  const positions = findAll(data, pattern);
  const padded = new Uint8Array(pattern.length);
  padded.set(replacement);

  for (const pos of positions) {
    for (let i = 0; i < padded.length; i++) {
      data[pos + i] = padded[i];
    }
  }
  return positions.length;
}

// Patch binary data with new domain
function patchBinary(data, newDomain) {
  const results = { ascii: 0, utf16: 0 };
  const newDomainAscii = toAscii(newDomain);
  const newDomainUtf16 = toUtf16Le(newDomain);

  // Replace each hostname with the new domain
  for (const hostname of HOSTNAMES) {
    // ASCII replacement
    const asciiPattern = toAscii(hostname);
    results.ascii += replaceAll(data, asciiPattern, newDomainAscii);

    // UTF-16LE replacement
    const utf16Pattern = toUtf16Le(hostname);
    results.utf16 += replaceAll(data, utf16Pattern, newDomainUtf16);
  }

  return results;
}

// Handle patcher form submission
function patchFile(event) {
  event.preventDefault();

  const file = document.getElementById('ggFile').files[0];
  const newDomain = document.getElementById('serverAddress').value.trim();
  const statusDiv = document.getElementById('patcherStatus');

  if (!file) {
    statusDiv.innerHTML = '<div class="info-box" style="background: #FFE0E0;">Nie wybrano pliku!</div>';
    return false;
  }

  if (!newDomain) {
    statusDiv.innerHTML = '<div class="info-box" style="background: #FFE0E0;">Podaj adres serwera!</div>';
    return false;
  }

  if (newDomain.length > MAX_DOMAIN_LENGTH) {
    statusDiv.innerHTML = '<div class="info-box" style="background: #FFE0E0;">Adres serwera max ' + MAX_DOMAIN_LENGTH + ' znakow! (masz: ' + newDomain.length + ')</div>';
    return false;
  }

  statusDiv.innerHTML = '<div class="info-box">Patchowanie w toku...</div>';

  const reader = new FileReader();
  reader.onload = function(e) {
    const data = new Uint8Array(e.target.result);
    const originalSize = data.length;

    const results = patchBinary(data, newDomain);
    const total = results.ascii + results.utf16;

    if (total === 0) {
      statusDiv.innerHTML = '<div class="info-box" style="background: #FFE0E0;">Nie znaleziono domen do zamiany. Czy to oryginalny plik GG?</div>';
      return;
    }

    if (data.length !== originalSize) {
      statusDiv.innerHTML = '<div class="info-box" style="background: #FFE0E0;">Blad: zmieniono rozmiar pliku!</div>';
      return;
    }

    const blob = new Blob([data], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const outputName = file.name.replace(/\.exe$/i, '') + '_patched.exe';

    statusDiv.innerHTML = `
      <div class="info-box" style="background: #E0FFE0;">
        Plik spatchowany!<br><br>
        <strong>Zamieniono:</strong><br>
        - Wszystkie domeny *.gadu-gadu.pl -> ${newDomain}<br>
        - ASCII: ${results.ascii}x, UTF-16: ${results.utf16}x<br><br>
        <a href="${url}" download="${outputName}" class="button">Pobierz ${outputName}</a>
      </div>
    `;
  };

  reader.onerror = function() {
    statusDiv.innerHTML = '<div class="info-box" style="background: #FFE0E0;">Blad odczytu pliku!</div>';
  };

  reader.readAsArrayBuffer(file);
  return false;
}
