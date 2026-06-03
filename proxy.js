// proxy.js — прокси-сервер с диагностикой
const http = require('http');
const fs = require('fs');
const path = require('path');

const FRONTEND_PORT = 3000;
const API_TARGET = 'http://localhost:8080';

// Показываем, откуда запускается сервер
console.log('📁 Рабочая папка:', __dirname);
console.log('📄 Содержимое папки:', fs.readdirSync(__dirname).join(', '));
console.log('📡 Проксируем API на:', API_TARGET);
console.log('');

const server = http.createServer((req, res) => {
    console.log(`\n➡️  ${req.method} ${req.url}`);

    // CORS-заголовки
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

    // Preflight
    if (req.method === 'OPTIONS') {
        console.log('   ✅ OPTIONS preflight');
        res.writeHead(204);
        res.end();
        return;
    }

    // Прокси для API
    if (req.url.startsWith('/api/')) {
        const apiUrl = API_TARGET + req.url.replace('/api', '');
        console.log(`   🔄 Проксирую на: ${apiUrl}`);

        const options = {
            method: req.method,
            headers: { ...req.headers, host: new URL(API_TARGET).host }
        };
        delete options.headers.origin;
        delete options.headers.referer;
        delete options.headers['content-length']; // пересчитается автоматически

        const proxyReq = http.request(apiUrl, options, (proxyRes) => {
            console.log(`   ⬅️  Ответ от API: ${proxyRes.statusCode}`);
            res.writeHead(proxyRes.statusCode, proxyRes.headers);
            proxyRes.pipe(res);
        });

        proxyReq.on('error', (err) => {
            console.error('   ❌ Ошибка прокси:', err.message);
            res.writeHead(502);
            res.end('API unreachable: ' + err.message);
        });

        req.pipe(proxyReq);
        return;
    }

    // Раздача статики
    let filePath = req.url === '/' ? '/index.html' : req.url;
    filePath = path.join(__dirname, filePath);
    console.log(`   📂 Ищу файл: ${filePath}`);

    const ext = path.extname(filePath);
    const mimeTypes = {
        '.html': 'text/html; charset=utf-8',
        '.js': 'text/javascript',
        '.css': 'text/css',
        '.png': 'image/png',
        '.json': 'application/json'
    };

    fs.readFile(filePath, (err, data) => {
        if (err) {
            console.log(`   ❌ Файл не найден: ${filePath}`);
            res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
            res.end(`Not found: ${req.url}\n\nОжидался файл: ${filePath}\n\nФайлы в папке:\n${fs.readdirSync(__dirname).join('\n')}`);
            return;
        }
        console.log(`   ✅ Отдаю файл (${data.length} байт)`);
        res.writeHead(200, { 'Content-Type': mimeTypes[ext] || 'text/plain' });
        res.end(data);
    });
});

server.listen(FRONTEND_PORT, () => {
    console.log(`\n✅ Сервер запущен: http://localhost:${FRONTEND_PORT}`);
    console.log(`👉 Откройте в браузере: http://localhost:${FRONTEND_PORT}\n`);
});