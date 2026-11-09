# GLASTOR® Core — Favicon & Brand Asset Suite

Suite completa de favicons, iconos vectoriales y manifiesto de aplicación para el ecosistema **GLASTOR® Core** y aplicaciones Rust WASM (`glastor-forge`).

---

## 📁 Estructura del Directorio

```text
public/favicon/
├── favicon.svg                 # Favicon vectorial SVG nativo (modo oscuro/claro)
├── favicon.ico                 # Fallback binario ICO multiresolución
├── favicon-16x16.png           # Pestañas de navegador clásicas (16px)
├── favicon-32x32.png           # Pestañas Retina y barras de marcadores (32px)
├── apple-touch-icon.png        # Icono iOS Home Screen (180x180)
├── android-chrome-192x192.png  # PWA Android pantalla de inicio (192x192)
├── android-chrome-512x512.png  # PWA Splash Screen y alta resolución (512x512)
├── site.webmanifest            # Manifiesto PWA Web App W3C
├── byte-gauge.svg              # Icono vectorial oficial del subsistema byte-gauge
├── specter.svg                 # Icono vectorial oficial del subsistema specter
├── zerho.svg                   # Icono vectorial oficial del subsistema zerho
└── build_favicons.ps1          # Script automatizado C# / .NET para regeneración de bitmaps
```

---

## 🚀 Integración en HTML

Para integrar esta suite en cualquier plantilla HTML (o en aplicaciones Leptos / Dioxus / Yew):

```html
<!-- Favicons & App Icons -->
<link rel="icon" type="image/svg+xml" href="public/favicon/favicon.svg">
<link rel="alternate icon" type="image/png" sizes="32x32" href="public/favicon/favicon-32x32.png">
<link rel="alternate icon" type="image/png" sizes="16x16" href="public/favicon/favicon-16x16.png">
<link rel="apple-touch-icon" sizes="180x180" href="public/favicon/apple-touch-icon.png">
<link rel="manifest" href="public/favicon/site.webmanifest">
<meta name="theme-color" content="#09090b">
```
