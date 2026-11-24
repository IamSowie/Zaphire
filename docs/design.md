# Zaphire: diseño

## Objetivo

Navegador para Windows y Linux que gaste poco cuando solo se lee y que dé todo
el rendimiento a una pestaña cuando se juega. Compatible con extensiones de
Chrome. Interfaz moderna y personalizable.

## Decisiones

| Tema | Decisión | Motivo |
| --- | --- | --- |
| Motor | Chromium vía CEF 156 (crate `cef` de tauri-apps) | Compatibilidad web total, WebGL/WebGPU, extensiones de Chrome |
| Interfaz | React + Tailwind v4 + Motion, servida en `zaphire://app/` | Personalizable, permite usar las librerías de UI elegidas |
| Pestañas | Verticales por defecto, horizontales opcional | Lo elegido por el usuario |
| Estilo | Oscuro con cristal, acento zafiro | Lo elegido por el usuario |
| Licencia | MPL-2.0 | Copyleft por archivo, compatible con el motor de Brave |

## Arquitectura

```
┌──────────────────────── ventana (CEF Views, sin marco) ────────────────────────┐
│ BrowserView UI (Alloy)          │  Panel de contenido (BoxLayout horizontal)  │
│ zaphire://app/                  │  ┌─ BrowserView pestaña activa (Alloy) ─┐  │
│ React: barra lateral/superior   │  │                                       │  │
│                                 │  └─ BrowserView vista dividida ─────────┘  │
└─────────────────────────────────┴─────────────────────────────────────────────┘
```

- **Todas las pestañas existen como BrowserView** dentro del panel; las que no
  se ven quedan ocultas (`set_visible(0)`), y Chromium deja de pintarlas.
- **Suspensión**: cada 30 s se cierran las pestañas que llevan más tiempo del
  que marca el perfil sin verse (salvo fijadas o con audio). Al activarlas se
  vuelven a crear desde su URL.
- **Estado**: `zaphire-core::Session` es la única fuente de verdad. La capa CEF
  solo guarda `TabId → BrowserView`.

### Comunicación UI ⇄ núcleo

- UI → núcleo: `fetch("zaphire://app/api", {method: "POST", body: Command})`.
  El manejador del esquema comprueba que la llamada venga de la vista UI o de
  una página `zaphire://app/…`, así que una web no puede dar órdenes.
- Núcleo → UI: `window.__zaphire.receive(Event)` vía `execute_java_script`,
  agrupado a un envío por fotograma (16 ms).
- Páginas en pestañas (nueva pestaña, ajustes) leen
  `zaphire://app/api/state` una vez por segundo mientras están visibles.

### Extensiones

CEF permite **una sola vista estilo Chrome por ventana**. Zaphire necesita
varias (UI + pestañas + vista dividida), así que la ventana principal usa
vistas **estilo Alloy**. Comprobado en esta versión:

| Qué | Pestañas Alloy | Ventana Chrome |
| --- | --- | --- |
| Content scripts de extensiones | ✅ funcionan | ✅ |
| `chrome://extensions` | ❌ queda en blanco | ✅ |
| Instalar desde Chrome Web Store | ❌ | ✅ |
| Botones/popup de la extensión | ❌ | ✅ (barra de Chrome) |

Por eso `chrome://extensions` y la Web Store se abren en una **ventana
estilo Chrome** aparte (`open_chrome_window`). Lo instalado ahí funciona en
todas las pestañas porque el perfil es el mismo.

Pendiente: mostrar los botones de las extensiones en la propia barra de
Zaphire (abrir `chrome-extension://<id>/<popup>` en una vista flotante).

### Perfiles de rendimiento

`zaphire-perf::Profile` define dos cosas:

1. **Switches de Chromium**, que solo se aplican al arrancar. Por eso la UI
   avisa con «Reiniciar para aplicar».
2. **Política en vivo**, que se aplica al instante: suspensión, congelar y
   silenciar las pestañas de fondo, y prioridad del proceso.

## Consumo medido (Linux, Xvfb, compilación debug, GPU por software)

Interfaz más 4 pestañas (una página local, nueva pestaña, ajustes y
`chrome://version`): **546 MB PSS** en total entre todos los procesos. El
proceso GPU con SwiftShader se lleva 110 MB, que con una GPU real son bastante
menos. Falta medir una compilación release en hardware real.

## Hoja de ruta

1. Botones de extensiones en la barra de Zaphire.
2. Historial, marcadores, descargas y restaurar la sesión al reabrir.
3. Paleta de comandos (Ctrl+K) con `@kobra/command-menu` o
   `spotlight-search` de EasyUI.
4. Descargar EasyList y EasyPrivacy desde Ajustes y actualizarlas solas.
5. Mica/acrílico real en Windows 11 (DWM) para el cristal.
6. Superposición de rendimiento (FPS/RAM) en Modo Juego.
7. Instaladores (MSI y AppImage) y actualizaciones automáticas.
