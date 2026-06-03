// ============================================================
// КОНФИГУРАЦИЯ API
// ============================================================
const API_BASE = '/api'; // Прокси будет перенаправлять на http://localhost:8080

// ============================================================
// STATE MANAGEMENT
// ============================================================
const state = {
    user: null,
    token: localStorage.getItem('ps_token') || null,
    isRegistering: false,
    uploadedImages: [],
    activeImageId: null,
    selectionPoints: [],
    album: [],
    tilesetItems: []
};

// ============================================================
// API HELPER
// ============================================================
async function apiRequest(endpoint, options = {}) {
    const headers = {
        'Content-Type': 'application/json',
        ...(options.headers || {})
    };

    if (state.token) {
        // Отправляем только чистый JWT-токен без "Bearer "
        headers['Authorization'] = state.token;

        console.log('Токен (длина):', state.token.length);
        console.log('Токен начинается с:', state.token.substring(0, 10));

        // Если токен начинается с кавычки, очищаем его
        if (state.token.startsWith('"') && state.token.endsWith('"')) {
            console.warn('Обнаружены лишние кавычки в токене, очищаем...');
            state.token = state.token.slice(1, -1);
            headers['Authorization'] = state.token;
            localStorage.setItem('ps_token', state.token);
        }
    }

    const url = `${API_BASE}${endpoint}`;
    console.log(`[API] ${options.method || 'GET'} ${url}`);

    try {
        const res = await fetch(url, { ...options, headers });

        if (!res.ok) {
            let errorMsg = `Ошибка ${res.status}`;
            try {
                const contentType = res.headers.get('content-type') || '';
                if (contentType.includes('application/json')) {
                    const data = await res.json();
                    errorMsg = data.message || data.error || JSON.stringify(data);
                } else {
                    const text = await res.text();
                    if (text) errorMsg = text;
                }
            } catch (e) { }

            if (res.status === 409) errorMsg = 'Email уже используется';
            else if (res.status === 401) errorMsg = 'Неверный email или пароль';
            else if (res.status === 502) errorMsg = 'Внешние сервисы недоступны';
            else if (res.status === 500) errorMsg = 'Внутренняя ошибка сервера';

            throw new Error(errorMsg);
        }
        return res;
    } catch (err) {
        if (err instanceof TypeError && err.message === 'Failed to fetch') {
            throw new Error(`Не удалось связаться с сервером. Проверьте, запущен ли API и прокси.`);
        }
        throw err;
    }
}

// ============================================================
// AUTH LOGIC
// ============================================================
const authView = document.getElementById('auth-view');
const appView = document.getElementById('app-view');
const authForm = document.getElementById('auth-form');
const toggleAuthBtn = document.getElementById('toggle-auth');
const authSubmitBtn = document.getElementById('auth-submit-btn');
const authStatusEl = document.getElementById('auth-status');

function renderAuth() {
    document.getElementById('auth-title').textContent = state.isRegistering ? 'Регистрация' : 'Вход в систему';
    document.getElementById('confirm-password-group').classList.toggle('hidden', !state.isRegistering);
    authSubmitBtn.textContent = state.isRegistering ? 'Зарегистрироваться' : 'Войти';
    toggleAuthBtn.textContent = state.isRegistering ? 'Уже есть аккаунт? Войти' : 'Нет аккаунта? Зарегистрироваться';
}

toggleAuthBtn.addEventListener('click', () => {
    state.isRegistering = !state.isRegistering;
    authStatusEl.textContent = '';
    authStatusEl.className = 'status-msg';
    renderAuth();
});

authForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const email = document.getElementById('email').value.trim();
    const password = document.getElementById('password').value;

    authStatusEl.textContent = '';
    authStatusEl.className = 'status-msg';
    authSubmitBtn.disabled = true;
    authSubmitBtn.innerHTML = '<span class="loader"></span>Обработка...';

    console.log('Начало процесса авторизации...');

    try {
        const endpoint = state.isRegistering ? '/create_user' : '/auth_user';
        console.log(`📡 Отправка запроса на: ${API_BASE}${endpoint}`);

        const res = await apiRequest(endpoint, {
            method: 'POST',
            body: JSON.stringify({ email, password })
        });

        console.log('Получен ответ от сервера. Статус:', res.status);

        let data;
        try {
            data = await res.json();
            console.log('Данные от сервера:', data);
        } catch (parseErr) {
            console.error('Ошибка парсинга JSON:', parseErr);
            throw new Error('Сервер вернул некорректные данные (не JSON)');
        }

        if (!data || !data.token) {
            console.error('В ответе сервера нет поля "token"!', data);
            throw new Error('Ошибка API: сервер не вернул токен. Проверьте формат ответа.');
        }

        console.log('Токен получен, сохраняем...');
        state.token = data.token;
        state.user = { email, token: data.token };
        localStorage.setItem('ps_token', data.token);
        localStorage.setItem('ps_email', email);

        console.log('Запуск приложения...');
        await initApp();

    } catch (err) {
        console.error('КРИТИЧЕСКАЯ ОШИБКА АВТОРИЗАЦИИ:', err);
        authStatusEl.textContent = err.message;
        authStatusEl.className = 'status-msg error';
    } finally {
        authSubmitBtn.disabled = false;
        authSubmitBtn.textContent = state.isRegistering ? 'Зарегистрироваться' : 'Войти';
    }
});

document.getElementById('logout-btn').addEventListener('click', () => {
    state.user = null;
    state.token = null;
    localStorage.removeItem('ps_token');
    localStorage.removeItem('ps_email');
    state.uploadedImages = [];
    state.album = [];
    state.tilesetItems = [];
    state.activeImageId = null;
    authView.style.display = 'flex';
    appView.style.display = 'none';
    renderAuth();
});

// ============================================================
// APP INITIALIZATION
// ============================================================
async function initApp() {
    if (!state.token) {
        authView.style.display = 'flex';
        appView.style.display = 'none';
        return;
    }

    try {
        console.log('Проверка токена через /check_token...');
        const res = await apiRequest('/check_token', { method: 'GET' });

        const responseText = await res.text();
        console.log('Ответ от /check_token:', responseText);

        if (responseText.includes('InvalidFormat') || responseText.includes('invalid')) {
            throw new Error('Токен недействителен или имеет неверный формат');
        }

    } catch (err) {
        console.warn('Токен невалиден, требуется повторный вход:', err.message);
        state.token = null;
        state.user = null;
        localStorage.removeItem('ps_token');
        localStorage.removeItem('ps_email');
        authView.style.display = 'flex';
        appView.style.display = 'none';
        renderAuth();
        return;
    }

    const email = localStorage.getItem('ps_email') || 'user';
    state.user = { email, token: state.token };
    authView.style.display = 'none';
    appView.style.display = 'flex';
    document.getElementById('user-email-display').textContent = email;

    state.uploadedImages = JSON.parse(localStorage.getItem('ps_images') || '[]');
    state.album = JSON.parse(localStorage.getItem('ps_album') || '[]');
    renderUploadedList();
    renderAlbum();
    if (state.activeImageId) loadActiveImage();
}

// ============================================================
// IMAGE UPLOAD & MANAGEMENT
// ============================================================
document.getElementById('image-upload').addEventListener('change', (e) => {
    const files = Array.from(e.target.files);
    files.forEach(file => {
        const reader = new FileReader();
        reader.onload = (event) => {
            const newImg = {
                id: Date.now() + Math.random(),
                name: file.name,
                src: event.target.result
            };
            state.uploadedImages.push(newImg);
            saveImages();
            renderUploadedList();
            setActiveImage(newImg.id);
        };
        reader.readAsDataURL(file);
    });
    e.target.value = '';
});

function saveImages() {
    localStorage.setItem('ps_images', JSON.stringify(state.uploadedImages));
}

function renderUploadedList() {
    const list = document.getElementById('uploaded-list');
    list.innerHTML = '';
    state.uploadedImages.forEach(img => {
        const div = document.createElement('div');
        div.className = `uploaded-item ${state.activeImageId == img.id ? 'active' : ''}`;
        div.innerHTML = `
                    <img src="${img.src}">
                    <span>${img.name}</span>
                    <span class="close-img" data-id="${img.id}">×</span>
                `;
        div.addEventListener('click', (e) => {
            if (e.target.classList.contains('close-img')) {
                state.uploadedImages = state.uploadedImages.filter(i => i.id != e.target.dataset.id);
                if (state.activeImageId == e.target.dataset.id) {
                    state.activeImageId = null;
                    document.getElementById('image-container').classList.add('hidden');
                    document.getElementById('empty-workspace-msg').classList.remove('hidden');
                }
                saveImages();
                renderUploadedList();
            } else {
                setActiveImage(img.id);
            }
        });
        list.appendChild(div);
    });
}

function setActiveImage(id) {
    state.activeImageId = id;
    state.selectionPoints = [];
    renderUploadedList();
    loadActiveImage();
}

function loadActiveImage() {
    const img = state.uploadedImages.find(i => i.id == state.activeImageId);
    if (!img) return;

    document.getElementById('empty-workspace-msg').classList.add('hidden');
    const container = document.getElementById('image-container');
    container.classList.remove('hidden');

    const imgEl = document.getElementById('active-image');
    imgEl.onload = () => {
        const canvas = document.getElementById('selection-canvas');
        canvas.width = imgEl.clientWidth;
        canvas.height = imgEl.clientHeight;
        drawSelection();
    };
    imgEl.src = img.src;
}

// ============================================================
// POLYGON SELECTION
// ============================================================
const canvas = document.getElementById('selection-canvas');
const ctx = canvas.getContext('2d');

canvas.addEventListener('click', (e) => {
    if (!state.activeImageId) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    state.selectionPoints.push({ x, y });
    drawSelection();
});

function drawSelection() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (state.selectionPoints.length === 0) return;

    ctx.beginPath();
    ctx.moveTo(state.selectionPoints[0].x, state.selectionPoints[0].y);
    for (let i = 1; i < state.selectionPoints.length; i++) {
        ctx.lineTo(state.selectionPoints[i].x, state.selectionPoints[i].y);
    }
    ctx.strokeStyle = '#e94560';
    ctx.lineWidth = 2;
    ctx.stroke();

    state.selectionPoints.forEach(p => {
        ctx.beginPath();
        ctx.arc(p.x, p.y, 4, 0, Math.PI * 2);
        ctx.fillStyle = '#ff6b6b';
        ctx.fill();
    });
}

document.getElementById('clear-selection-btn').addEventListener('click', () => {
    state.selectionPoints = [];
    drawSelection();
});

// ============================================================
// GENERATION
// ============================================================
document.getElementById('close-contour-btn').addEventListener('click', () => {
    if (state.selectionPoints.length < 3) {
        alert('Поставьте минимум 3 точки для создания контура');
        return;
    }

    const imgEl = document.getElementById('active-image');
    const tempCanvas = document.createElement('canvas');
    const scaleX = imgEl.naturalWidth / imgEl.clientWidth;
    const scaleY = imgEl.naturalHeight / imgEl.clientHeight;

    tempCanvas.width = imgEl.naturalWidth;
    tempCanvas.height = imgEl.naturalHeight;
    const tCtx = tempCanvas.getContext('2d');

    tCtx.beginPath();
    tCtx.moveTo(state.selectionPoints[0].x * scaleX, state.selectionPoints[0].y * scaleY);
    for (let i = 1; i < state.selectionPoints.length; i++) {
        tCtx.lineTo(state.selectionPoints[i].x * scaleX, state.selectionPoints[i].y * scaleY);
    }
    tCtx.closePath();
    tCtx.clip();
    tCtx.drawImage(imgEl, 0, 0);

    const targetRes = parseInt(document.getElementById('resolution').value);
    const colorCount = parseInt(document.getElementById('color-count').value);
    const outlineCount = parseInt(document.getElementById('outline-count').value);

    const finalDataUrl = pixelateAndQuantize(tempCanvas, targetRes, targetRes, colorCount, outlineCount);

    const sprite = {
        id: Date.now(),
        src: finalDataUrl,
        type: document.getElementById('object-type').value,
        resolution: targetRes
    };

    state.album.push(sprite);
    localStorage.setItem('ps_album', JSON.stringify(state.album));

    state.selectionPoints = [];
    drawSelection();
    renderAlbum();
});

function pixelateAndQuantize(sourceCanvas, targetW, targetH, colorCount, outlineCount) {
    const temp = document.createElement('canvas');
    temp.width = targetW;
    temp.height = targetH;
    const tCtx = temp.getContext('2d');
    tCtx.imageSmoothingEnabled = false;
    tCtx.drawImage(sourceCanvas, 0, 0, targetW, targetH);

    const imgData = tCtx.getImageData(0, 0, targetW, targetH);
    const data = imgData.data;

    if (colorCount > 0) {
        const levels = Math.ceil(Math.pow(colorCount, 1 / 3));
        const step = 256 / levels;
        for (let i = 0; i < data.length; i += 4) {
            if (data[i + 3] < 128) continue;
            data[i] = Math.floor(data[i] / step) * step + step / 2;
            data[i + 1] = Math.floor(data[i + 1] / step) * step + step / 2;
            data[i + 2] = Math.floor(data[i + 2] / step) * step + step / 2;
        }
    }

    if (outlineCount > 0) {
        const copy = new Uint8ClampedArray(data);
        for (let y = 1; y < targetH - 1; y++) {
            for (let x = 1; x < targetW - 1; x++) {
                const idx = (y * targetW + x) * 4;
                const rightIdx = idx + 4;
                if (data[idx + 3] < 128) continue;

                const diff = Math.abs(data[idx] - data[rightIdx]) + Math.abs(data[idx + 1] - data[rightIdx + 1]) + Math.abs(data[idx + 2] - data[rightIdx + 2]);
                if (diff > 40) {
                    copy[idx] = 0; copy[idx + 1] = 0; copy[idx + 2] = 0;
                }
            }
        }
        for (let i = 0; i < data.length; i++) data[i] = copy[i];
    }

    tCtx.putImageData(imgData, 0, 0);
    return temp.toDataURL();
}

// ============================================================
// ALBUM & TILESET
// ============================================================
function renderAlbum() {
    const grid = document.getElementById('album-grid');
    const emptyMsg = document.getElementById('album-empty');
    grid.innerHTML = '';

    if (state.album.length === 0) {
        emptyMsg.style.display = 'block';
    } else {
        emptyMsg.style.display = 'none';
        state.album.forEach(item => {
            const div = document.createElement('div');
            div.className = 'album-item';
            div.innerHTML = `
                        <img src="${item.src}">
                        <div class="tools">
                            <button class="tool-btn" title="В тайлсет" onclick="addToTileset(${item.id})"></button>
                            <button class="tool-btn delete" title="Удалить" onclick="deleteFromAlbum(${item.id})">🗑</button>
                        </div>
                    `;
            grid.appendChild(div);
        });
    }
}

window.deleteFromAlbum = (id) => {
    state.album = state.album.filter(i => i.id !== id);
    state.tilesetItems = state.tilesetItems.filter(i => i.id !== id);
    localStorage.setItem('ps_album', JSON.stringify(state.album));
    renderAlbum();
    renderTileset();
};

window.addToTileset = (id) => {
    const item = state.album.find(i => i.id === id);
    if (item && !state.tilesetItems.find(i => i.id === id)) {
        state.tilesetItems.push(item);
        renderTileset();
    }
};

function renderTileset() {
    const cvs = document.getElementById('tileset-canvas');
    const tCtx = cvs.getContext('2d');
    tCtx.clearRect(0, 0, cvs.width, cvs.height);

    const cols = 4;
    const size = 64;

    state.tilesetItems.forEach((item, index) => {
        const x = (index % cols) * size;
        const y = Math.floor(index / cols) * size;

        if (y + size > cvs.height) {
            cvs.height = y + size;
        }

        const img = new Image();
        img.src = item.src;
        img.onload = () => {
            tCtx.imageSmoothingEnabled = false;
            tCtx.drawImage(img, x, y, size, size);
        };
    });
}

document.getElementById('generate-tileset-btn').addEventListener('click', () => {
    state.tilesetItems = [...state.album];
    renderTileset();
});

document.getElementById('download-album-btn').addEventListener('click', () => {
    const size = 128;
    const cols = 4;
    const rows = Math.ceil(state.album.length / cols);
    const cvs = document.createElement('canvas');
    cvs.width = cols * size;
    cvs.height = rows * size;
    const ctx = cvs.getContext('2d');

    let loaded = 0;
    if (state.album.length === 0) return;
    state.album.forEach((item, i) => {
        const img = new Image();
        img.src = item.src;
        img.onload = () => {
            ctx.imageSmoothingEnabled = false;
            ctx.drawImage(img, (i % cols) * size, Math.floor(i / cols) * size, size, size);
            loaded++;
            if (loaded === state.album.length) {
                const link = document.createElement('a');
                link.download = 'pixel-album.png';
                link.href = cvs.toDataURL();
                link.click();
            }
        };
    });
});

document.getElementById('download-tileset-btn').addEventListener('click', () => {
    const link = document.createElement('a');
    link.download = 'tileset.png';
    link.href = document.getElementById('tileset-canvas').toDataURL();
    link.click();
});

window.addEventListener('resize', () => {
    if (state.activeImageId) {
        const imgEl = document.getElementById('active-image');
        const canvas = document.getElementById('selection-canvas');
        canvas.width = imgEl.clientWidth;
        canvas.height = imgEl.clientHeight;
        drawSelection();
    }
});

// ============================================================
// STARTUP
// ============================================================
renderAuth();
initApp();
