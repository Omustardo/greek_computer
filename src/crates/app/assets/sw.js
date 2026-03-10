// TEMPLATE_TODO: change my-app to your app name.
var cacheName = 'egui-my-app';

var filesToCache = [
    './',
    './index.html',
    // TEMPLATE_TODO: Change the instances of my_app to your app name.
    './my_app.js',
    './my_app_bg.wasm',
];

/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function (e) {
    e.waitUntil(
        caches.open(cacheName).then(function (cache) {
            return cache.addAll(filesToCache);
        })
    );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function (e) {
    e.respondWith(
        caches.match(e.request).then(function (response) {
            return response || fetch(e.request);
        })
    );
});
