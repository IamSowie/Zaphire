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

**Botones en la barra de Zaphire.** El runtime de Chrome no ofrece una API
para listar extensiones, así que Zaphire lee `extensions.settings` del
`Preferences` del perfil, abre el `manifest.json` de cada una y saca nombre,
icono y `action.default_popup` (`zaphire-core::extensions`). Chromium escribe
ese archivo con retraso, por eso la lista se vuelve a leer cada 20 s y al
cerrar la ventana de extensiones.

Al pulsar un botón, el popup (`chrome-extension://<id>/<popup>`) se abre en
una BrowserView Alloy **flotante** (`Window::add_overlay_view`) que se ajusta a
su contenido (`set_auto_resize_enabled` + `on_auto_resize`). Comprobado con una
extensión de prueba: el popup carga y `chrome.storage` funciona. Se cierra al
volver el foco a la página o a la interfaz.

Limitación: las APIs que dependen de las pestañas de Chrome
(`chrome.tabs.query` para la pestaña actual, `activeTab`) no ven las pestañas
Alloy de Zaphire, así que las extensiones que actúan sobre «la pestaña actual»
desde su popup pueden no funcionar del todo.

### Perfiles de rendimiento

`zaphire-perf::Profile` define dos cosas:

1. **Switches de Chromium**, que solo se aplican al arrancar. Por eso la UI
   avisa con «Reiniciar para aplicar».
2. **Política en vivo**, que se aplica al instante: suspensión, congelar y
   silenciar las pestañas de fondo, y prioridad del proceso.

## Personalización

El tema vive en `Settings.theme` (`zaphire-core::settings`):

- `mode`: oscuro, claro o sistema.
- `background`: degradado o plano.
- `background_color`: color base propio.
- `accent`, `glass`, `radius`.
- `scale`: zoom de la interfaz.
- `density` y `reduce_motion`.

La interfaz lo aplica con atributos en `<html>` (`data-theme`, `data-bg`,
`data-density`) y variables CSS (`ui/src/index.css`). Todos los colores salen
de tokens: `--z-ink` es el color de las superposiciones (blanco en oscuro,
pizarra en claro), así los paneles funcionan sobre cualquier fondo. El núcleo
pinta la ventana y el hueco de la vista dividida con
`Theme::frame_color()`, para que no aparezca una franja oscura en el tema
claro. Con «sistema», la interfaz avisa al núcleo cuando el sistema cambia
entre claro y oscuro.

## Historial, marcadores y sesión

`zaphire-core::library` guarda `history.json` (máximo 5000 visitas, las
recargas se funden con la visita anterior), `bookmarks.json` y
`session.json` en la carpeta de configuración. Se escriben cada 20 s si hubo
cambios y al cerrar. La sesión restaurada marca todas las pestañas como
suspendidas: solo cargan las que se ven.

## Instaladores

| Sistema | Formato | Cómo |
| --- | --- | --- |
| Linux | AppImage + tar.gz | `scripts/appimage.sh` (appimagetool, sin FUSE) |
| Windows | Instalador `.exe` (Inno Setup, por usuario) + zip | `packaging/windows/zaphire.iss` en la CI |

Una etiqueta `v*` lanza el trabajo `release` de la CI, que publica todo en
GitHub Releases.

## Consumo medido (Linux, Xvfb, compilación debug, GPU por software)

Interfaz más 4 pestañas (una página local, nueva pestaña, ajustes y
`chrome://version`): **546 MB PSS** en total entre todos los procesos. El
proceso GPU con SwiftShader se lleva 110 MB, que con una GPU real son bastante
menos. Falta medir una compilación release en hardware real.

## Hoja de ruta

1. Descargas con panel propio; carpetas de marcadores e importar desde Chrome/Firefox.
2. Paleta de comandos (Ctrl+K) con `@kobra/command-menu` o
   `spotlight-search` de EasyUI.
3. Descargar EasyList y EasyPrivacy desde Ajustes y actualizarlas solas.
4. Mica/acrílico real en Windows 11 (DWM) para el cristal.
5. Superposición de rendimiento (FPS/RAM) en Modo Juego.
6. Actualizaciones automáticas.
