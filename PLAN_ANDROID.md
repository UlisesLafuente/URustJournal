# Plan: UJournal para Android (APK)

Estado: implementado para la versión 0.5.4. El APK compila con GameActivity + Gradle + `cargo-ndk` y se validó en un emulador Android 15 (x86_64); incluye los backends wgpu Vulkan y GLES, un parche temporal de IME de Android sobre winit 0.30 y persistencia de preferencias de UI.
Objetivo: publicar un APK instalable que conserve toda la funcionalidad actual de UJournal y ofrezca una interfaz cómoda en una pantalla táctil vertical de teléfono.

---

## 1. Resumen

UJournal ya separa razonablemente bien dominio, almacenamiento y UI:

- Dominio y formato de archivos: `src/journal/*`.
- Almacenamiento en disco: `src/journal/storage.rs`.
- Estado y toda la UI: `src/app.rs` (monolítico, ~1.500 líneas).
- Entrada de escritorio: `src/main.rs`.
- Temas: `src/theme.rs`; idiomas: `src/i18n.rs`.

El trabajo Android tiene dos frentes:

1. **Port técnico**: compilar eframe/egui para Android, resolver el directorio de datos, las fuentes y el ciclo de vida de la app.
2. **Rediseño de interfaz**: sustituir paneles laterales, menús con clic derecho, ventanas flotantes y atajos de teclado por navegación de pantalla completa, hojas inferiores, pulsación larga, botones grandes y un compositor fijo de anotaciones.

La lógica de negocio (journals, secciones, páginas, bullets, colores, serialización, autoguardado) se reutiliza sin cambios. El formato `.txt` se mantiene para que los journals sigan siendo portables entre escritorio y Android.

### Garantía para macOS, Linux y Windows

**El trabajo Android no cambia el comportamiento ni la versión de las builds de escritorio.** Linux, macOS y Windows seguirán desarrollándose y publicándose como hasta ahora.

- Todo lo específico de Android queda aislado con `#[cfg(target_os = "android")]`, features de Cargo por plataforma y el directorio `android/`.
- Los flujos y binarios de escritorio conservan sus features actuales (`accesskit`, `glow`, `x11`, `wayland`), su UI y sus workflows de release.
- La refactorización de `src/app.rs` es organizativa: la UI de escritorio se mueve a un módulo, pero mantiene el mismo comportamiento y se cubre con los tests existentes.
- El formato de datos y la ruta de escritorio no cambian; no hay migraciones que afecten a instalaciones actuales.
- Cualquier regresión de escritorio se considera un fallo bloqueante del port Android.

---

## 2. Decisiones recomendadas

| Tema | Recomendación |
|------|---------------|
| Framework | Mantener eframe/egui 0.33: `wgpu` en Android (GameActivity) y `glow` en escritorio. |
| Entrada Android | `winit` + `android-activity`, usando la feature `android-game-activity` de eframe con Gradle. Implementado. |
| Estrategia de build | `android-game-activity` + Gradle + `cargo-ndk`, para conservar AccessKit/TalkBack y controlar el ciclo de vida y el botón atrás desde el inicio. |
| Almacenamiento | Directorio interno privado de la app (`AndroidApp::internal_data_path()`), sin permisos de almacenamiento. Mantener el mismo formato y nombres de archivo. |
| Fuentes | Mantener Caveat embebida. En Android, `load_cjk_font()` no puede leer fuentes del sistema: empaquetar una fuente CJK (Noto Sans JP) o desactivar el fallback CJK en esa plataforma. |
| Orientación | Diseñar para vertical; permitir rotación sin romper el layout. Bloquear a vertical solo si QA detecta problemas graves. |
| Distribución | APK universal para instalación directa y, opcionalmente, APK por ABI para reducir tamaño. Firma de release con keystore en secretos de CI. |
| Escritorio | La UI actual se conserva. La app elige layout móvil/escritorio en tiempo de ejecución según el ancho y/o `cfg!(target_os = "android")`. |

---

## 3. Brechas actuales frente a Android

| Área | Situación actual | Problema en Android | Cambio necesario |
|------|------------------|---------------------|------------------|
| Entrada | `fn main()` + `eframe::run_native` | Android necesita `android_main(app: AndroidApp)` y `NativeOptions.android_app`. | Añadir entrada Android y `crate-type = ["cdylib", "rlib"]`. |
| Datos | `ProjectDirs` (`src/journal/storage.rs`) | `directories` no garantiza una ruta válida en Android. | Resolver ruta con `internal_data_path()` y pasarla a `JournalStorage::with_dir`. |
| Fuentes CJK | Busca rutas de Linux/macOS/Windows en `load_cjk_font()` | Esas rutas no existen en Android. | Empaquetar Noto Sans JP o limitar CJK a escritorio. |
| Layout | `SidePanel` izquierdo + `TopBottomPanel` + panel central | Los paneles laterales y el ratón no encajan en un teléfono. | Nueva UI móvil de una columna con navegación por pantallas. |
| Acciones | Clic derecho, hover, menús `⋯`, atajos `↑↓/Enter/Esc` | No hay clic derecho ni hover; el teclado físico no siempre está. | Pulsación larga, botones visibles, hojas inferiores y botones de acción explícitos. |
| Diálogos | `egui::Window` flotante para crear/renombrar/About | Ventanas pequeñas y arrastrables, malas en móvil. | `egui::Modal` o hoja inferior a ancho completo. |
| Página | Barra de páginas con `x` pequeño al lado | `x` diminuta y fácil de pulsar por error. | Chips con menú por pulsación larga/`⋯` y confirmación al borrar. |
| Editor | Scroll anidado (página + bullets) y fila horizontal de añadir | Gestos de scroll compiten y el compositor queda incómodo. | Un solo scroll vertical y compositor fijo en el panel inferior. |
| Autoguardado | Guarda al final del frame con `dirty` | Android puede pausar/matar la app; conviene forzar guardado al perder foco. | Guardar también en pausa/pérdida de foco y limitar frecuencia. |
| Ventana | Tamaño inicial 960×640, min 640×400, icono generado | En móvil el viewport lo define el sistema. | Opciones específicas por plataforma; sin tamaño mínimo. |
| Empaquetado | Workflow de Linux/Windows/macOS | No hay APK ni firma Android. | Nuevo job de Android en CI y artefacto en el release. |

---

## 4. Fases de trabajo

### Fase 0 — Preparación del entorno

- Instalar Android SDK, NDK y JDK; definir `ANDROID_HOME` y `NDK_HOME`.
- Añadir targets Rust: `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android` (emulador).
- Instalar `cargo-ndk` y preparar un proyecto Gradle mínimo con GameActivity.
- Verificar un proyecto eframe mínimo en emulador antes de tocar UJournal.
- Documentar versiones mínimas: API 26 (Android 8) como objetivo inicial; validar si eframe/winit exige una API superior.

Criterio de salida: APK de prueba con una ventana egui que muestra texto y responde al tacto.

### Fase 1 — Shell Android de UJournal

- Añadir `crate-type = ["cdylib", "rlib"]` al `[lib]`.
- Crear entrada Android (`#[cfg(target_os = "android")] fn android_main(app: AndroidApp)`) que construya `JournalStorage` con la ruta interna y llame a `eframe::run_native`.
- En `NativeOptions`, rellenar `android_app`, eliminar tamaños mínimos y ajustar el viewport a pantalla completa.
- Configurar features de eframe por plataforma:
  - Escritorio: mantener `accesskit`, `glow`, `default_fonts`, `x11`, `wayland`.
  - Android: `accesskit`, `glow`, `default_fonts` y `android-game-activity`.
  - No usar `android-native-activity`: eframe no permite combinarla con `accesskit`.
- Añadir metadatos Android (package `org.ujournal.UJournal`, label `UJournal`, versión, icono, `minSdk`, `targetSdk`).

Criterio de salida: APK que abre UJournal, lista journals y permite crear/abrir uno.

### Fase 2 — Plataforma: datos, fuentes y ciclo de vida

- Añadir un módulo `src/platform/` o `src/android.rs` que resuelva:
  - `storage_root()`: `ProjectDirs` en escritorio; `internal_data_path()/journals` en Android; `with_dir` en tests.
  - Fallback controlado si `internal_data_path()` devuelve `None`.
- Empaquetar Noto Sans JP (licencia OFL) y registrar la familia `jp` también en Android; actualizar `assets/THIRD_PARTY_LICENSES.txt`.
- Guardar al perder foco (`viewport().focused == false`) y al salir (`on_exit`), además del autoguardado actual.
- Manejar el botón atrás de Android:
  - winit lo expone como `egui::Key::BrowserBack`; tratarlo como “atrás” de la UI.
  - Orden: cerrar modal/hoja → cerrar compositor/teclado → volver a la lista de journals → salir.
  - Evaluar `enableOnBackInvokedCallback`/predictive back si se usa un wrapper Gradle.
- Reducir repintados innecesarios y agrupar escrituras de autoguardado (por ejemplo, mínimo 300–500 ms entre guardados) para ahorrar batería.

Criterio de salida: los journals persisten tras cerrar la app, el texto japonés se ve correctamente y el botón atrás navega sin cerrar la app de golpe.

### Fase 3 — UI móvil (núcleo)

- Introducir `LayoutMode::{Desktop, Phone}` calculado con el ancho lógico (`ctx.screen_rect().width()`): `< 600–640 dp` ⇒ Phone; Android siempre Phone.
- Separar la UI en módulos para no duplicar lógica:
  - `src/app/mod.rs`: estado, acciones y `eframe::App`.
  - `src/app/desktop.rs`: la UI actual movida tal cual.
  - `src/app/mobile.rs`: nueva UI táctil.
  - `src/app/widgets.rs`: fuentes, colores, botones y componentes compartidos.
- Mantener una única fuente de verdad para las acciones (`create_journal`, `open_journal`, `create_page`, `add_bullet`, etc.) y hacer que ambas UIs solo las invoquen.

Criterio de salida: paridad funcional completa en móvil: journals, secciones, páginas, bullets, marcadores, colores, edición, borrado, movimiento, temas, idioma, About y autoguardado.

### Fase 4 — Diseño táctil y ergonomía

- Sustituir `SidePanel` por una pantalla/drawer de journals.
- Sustituir menús de clic derecho por pulsación larga (`Response::long_touched()`) y botones `⋯` visibles.
- Sustituir ventanas flotantes por `egui::Modal` o hojas inferiores.
- Composer fijo de anotaciones con selector de tipo, campo de texto y botón Añadir.
- Objetivos táctiles de al menos 48×48 dp; tipografía móvil mayor que la de escritorio.
- Respetar safe areas (notch, barra de gestos) y el teclado virtual.
- Un solo scroll vertical por pantalla; evitar scroll anidado.
- Confirmación explícita antes de borrar journal, página o anotación.

Criterio de salida: uso completo con una mano en un teléfono de 360–430 dp de ancho, sin acciones ocultas tras hover ni clic derecho.

### Fase 5 — Empaquetado y firma

- Usar Gradle + `cargo-ndk` + GameActivity, con firma release y artefactos por ABI/universal.
- Generar iconos Android en mipmaps (reutilizar `assets/app-icon.png`) y, opcionalmente, icono adaptativo.
- Firmar release con keystore propia; nunca subir la keystore al repositorio.
- Producir:
  - `UJournal-<version>-arm64-v8a.apk` (dispositivos reales).
  - `UJournal-<version>-x86_64.apk` (emuladores), si se desea.
  - APK universal como opción simple de instalación.
- Mapear `versionName` desde `Cargo.toml` y `versionCode` creciente y estable.

Criterio de salida: APK firmado instalable con `adb install` en un dispositivo real, sin permisos innecesarios.

### Fase 6 — CI y release

- Nuevo workflow `.github/workflows/android.yml`:
  - Instalar NDK y targets.
  - Compilar APK debug en pull requests.
  - Compilar APK release en tags `v*`.
  - Subir artefacto y adjuntarlo al GitHub Release junto a Linux/Windows/macOS.
- Cachear Cargo y Gradle.
- Verificar firma (`apksigner verify`) y tamaño del APK.
- Mantener el workflow de escritorio intacto.

Criterio de salida: tag `vX.Y.Z` publica automáticamente APK Android además de los binarios actuales.

### Fase 7 — QA

- Emulador: API 26, 30, 34/35; pantalla pequeña y grande.
- Dispositivo real: gama baja y gama media.
- Casos: rotación, multitarea, volver desde segundo plano, teclado abierto, cambio de tema/idioma, fuente del sistema grande, modo oscuro, TalkBack (si AccessKit está activo), borrado accidental, muchos bullets y páginas.
- Revisar `adb logcat` para crashes/ANR y consumo excesivo de batería.

---

## 5. Diseño de interfaz para teléfono vertical

### 5.1 Principios

1. **Una columna, una tarea**: cada pantalla responde a “dónde estoy y qué puedo hacer aquí”.
2. **Todo visible o a un toque**: nada depende de hover ni de clic derecho.
3. **Pulgar primero**: acciones principales en la mitad inferior; navegación y contexto arriba.
4. **Toque seguro**: áreas de 48×48 dp, separación de 8 dp, destructivos con confirmación.
5. **Menos cromo**: el contenido ocupa la mayor parte de la pantalla; los paneles colapsables desaparecen.
6. **Continuidad**: el estado (journal, sección, página) se conserva al rotar o volver de segundo plano.

### 5.2 Mapa de navegación

```
┌──────────────────────────────────┐
│ ☰   UJournal            ⋯        │  barra superior compacta
├──────────────────────────────────┤
│ [Future Log] [Monthly] [Journal] │  selector de sección
├──────────────────────────────────┤
│ (Hoy) (Semana 45) (+)            │  chips de páginas (scroll horizontal)
├──────────────────────────────────┤
│  •  Comprar pan              ⋯   │
│  x  Llamar al fontanero      ⋯   │
│  -  Idea para el proyecto    ⋯   │
│  o  Cumpleaños de Ana        ⋯   │
│                                  │
│  (scroll vertical de la página)  │
├──────────────────────────────────┤
│ [•][-][o]  Escribe una nota… [＋]│  compositor fijo
└──────────────────────────────────┘
```

Pantalla de journals (drawer o pantalla completa):

```
┌──────────────────────────────────┐
│ ☰   Journals            ⋯        │
├──────────────────────────────────┤
│  MiDiario                    ⋯   │
│  Trabajo                     ⋯   │
│  Ideas                       ⋯   │
│                                  │
│                                  │
│         [ + Nuevo journal ]      │
└──────────────────────────────────┘
```

### 5.3 Correspondencia escritorio → móvil

| Elemento actual | Comportamiento móvil |
|-----------------|----------------------|
| `SidePanel` de journals | Pantalla de journals a la que se entra con ☰; también se abre al iniciar sin journal. |
| Clic en journal | Abre el journal y vuelve al editor. |
| Clic derecho en journal / fondo | Pulsación larga o `⋯` abre hoja: Abrir, Renombrar, Borrar, Nuevo journal. |
| `+`/campo de crear journal en barra superior | Botón “Nuevo journal” al final de la lista + hoja de creación. |
| Flechas de colapsar paneles | Se eliminan en móvil; la navegación sustituye el plegado. |
| Pestañas de sección | Selector segmentado de ancho completo; 3 botones grandes. |
| Barra de páginas con `x` | Chips horizontales; `+` al final; borrar/renombrar desde hoja al pulsar larga o `⋯` del chip. |
| Campo “nueva página” | Hoja “Nueva página” con campo de texto y botón Crear. |
| Tarjeta de anotaciones con scroll propio | Sin tarjeta ni scroll interno: las anotaciones forman parte del scroll único de la página. |
| Clic en marcador → menú | Pulsar el marcador abre hoja de marcadores (6 opciones grandes con símbolo y nombre). |
| `⋯` de anotación | Botón `⋯` de 48 dp al final de la fila; abre hoja: Editar, Mover, Color, Borrar. |
| Clic derecho en texto | Pulsación larga sobre la fila abre la misma hoja de acciones. |
| Modo mover con `↑↓/Enter/Esc` | MVP: hoja “Mover” con “Subir”, “Bajar” y “Mover al principio/final”; fase posterior: arrastrar con asa. |
| Fila horizontal “Añadir tarea/nota/evento + campo” | Compositor fijo inferior: selector de tipo, campo de una línea o multilínea y botón Añadir. |
| Ventana Renombrar/Crear journal | Hoja inferior o `Modal` a ancho completo con campo, Crear/Guardar y Cancelar. |
| About | Pantalla u hoja a pantalla completa con scroll. |
| Submenú de temas | Pantalla de Ajustes con lista de temas y muestra de color; búsqueda opcional. |
| Submenú de idioma | Pantalla de Ajustes, selector de idioma. |
| Mensajes de estado/error en la barra superior | Snackbar/toast sobre el compositor; errores con acción de cierre. |
| Guardado automático | Igual, más guardado al perder foco; indicador discreto solo si hay error. |
| Atajos de teclado | Se conservan en escritorio; en móvil no se muestran ayudas de teclado. |

### 5.4 Componentes móviles

- **Barra superior (56 dp)**: ☰ (journals), título del journal actual, `⋯` (Ajustes/About). Tocar el título puede abrir la lista de journals.
- **Selector de sección (44–48 dp)**: tres botones segmentados; el activo con color de acento.
- **Tira de páginas (44 dp)**: `ScrollArea::horizontal().drag_to_scroll(true)`; chips de al menos 44 dp de alto; `+` fijo al final.
- **Filas de anotación (mín. 48 dp)**:
  - Marcador a la izquierda como botón grande.
  - Texto en el centro; un toque entra en edición en línea.
  - `⋯` a la derecha.
  - Fondo alterno o línea de cuaderno ya existente, simplificada para no sobrecargar.
- **Compositor (mín. 56 dp, crece con el texto)**:
  - Chips Task/Note/Event.
  - Campo de texto con placeholder.
  - Botón `＋` de 48 dp.
  - Se coloca en `TopBottomPanel::bottom`, de modo que suba con el teclado.
- **Hoja inferior**:
  - Fondo con esquinas superiores redondeadas, ancho completo, altura según contenido, máximo 70–80 % de la pantalla.
  - Lista de acciones con separadores y altura mínima 48 dp.
  - Botón Cancelar visible.
- **Modal de formulario**:
  - Título, campo, mensaje de error, botones Guardar/Cancelar a ancho completo.
  - Foco automático en el campo al abrir.
- **Snackbar**: mensaje breve con duración de 3–4 s, por encima del compositor; errores persistentes hasta descartar.

### 5.5 Estados vacíos y de error

- Sin journals: pantalla de bienvenida con explicación breve y botón “Crear mi primer journal”.
- Journal sin páginas: mensaje dentro de la sección + botón “Crear página”.
- Página vacía: texto guía y compositor enfocado.
- Sección vacía: mensaje y acceso a crear página.
- Error de nombre inválido o duplicado: se muestra dentro de la hoja, sin cerrarla.
- Pérdida de foco con cambios pendientes: guardar antes de salir; si falla, mostrar error persistente.

### 5.6 Teclado, IME y safe areas

- Usar `TopBottomPanel` para barra superior y compositor; el contenido central se ajusta solo.
- Aplicar los insets de safe area a barras y compositor (notch, indicador de gestos).
- No cerrar el teclado al tocar dentro del editor salvo que se pulse fuera del campo o una acción explícita.
- El campo de anotación debe permitir texto largo; el botón Añadir permanece visible.
- Evitar que `request_focus()` se ejecute en cada frame: pedir foco una sola vez al abrir la edición (el código actual lo hace mientras el campo no tiene foco y no hay puntero pulsado; en móvil esto puede robar el foco al teclado).
- Probar con teclados Gboard, SwiftKey y un teclado físico/emulador.

### 5.7 Accesibilidad móvil

- Mantener objetivos de 48 dp y etiquetas semánticas (`widget_info`, `labelled_by`).
- Usar `android-game-activity` + Gradle para mantener AccessKit y exponer semántica a TalkBack desde el inicio.
- Respetar el escalado de fuente del sistema en la medida en que egui lo permita; probar con fuente al 130–200 %.
- No depender solo del color para estados; usar símbolos y texto.

### 5.8 Tabletas y horizontal

- Teléfono vertical: layout principal descrito.
- Horizontal y tabletas: mantener una columna centrada con ancho máximo (~720 dp) para evitar líneas de texto larguísimas; opcionalmente mostrar journals como panel lateral si el ancho supera 900 dp.
- No es requisito inicial un layout de dos columnas; sí que nada se rompa al rotar.

---

## 6. Arquitectura de código propuesta

```
src/
  main.rs                 # entrada escritorio (sin cambios grandes)
  lib.rs                  # módulos + entrada android_main (cfg)
  app/
    mod.rs                # UJournalApp, estado, acciones, eframe::App
    desktop.rs            # UI actual (mover desde app.rs)
    mobile.rs             # UI táctil nueva
    widgets.rs            # fuentes, estilos, marcadores, hojas, modales
  platform/
    mod.rs                # storage_root(), back navigation, cfg
    android.rs            # internal_data_path, ciclo de vida, JNI si hace falta
    desktop.rs            # ProjectDirs
  journal/                # sin cambios funcionales
  theme.rs
  i18n.rs
```

Recomendaciones:

- No duplicar reglas de negocio en `mobile.rs`; llamar a los métodos existentes de `UJournalApp`.
- Añadir estado de navegación móvil: `MobileScreen::{Journals, Journal}`, `drawer_open`, `overlay: Option<MobileOverlay>`.
- Mantener los diálogos de crear/renombrar como estado compartido (`creating_journal`, `renaming_journal`, `rename_name`) y renderizarlos de forma distinta según el layout.
- Centralizar tamaños táctiles y tipografía móvil en `widgets.rs`/estilo.
- Extender `JournalStorage` solo con constructores/rutas; no cambiar el formato.

---

## 7. Cambios concretos por archivo

| Archivo | Cambio |
|---------|--------|
| `Cargo.toml` | `[lib] crate-type`; features de eframe por target (`android-game-activity` en Android). |
| `src/main.rs` | Mantener escritorio; extraer creación de `NativeOptions` para reutilizar en Android. |
| `src/lib.rs` | Declarar `platform`; añadir `android_main` bajo `cfg(target_os = "android")`. |
| `src/journal/storage.rs` | Añadir constructor para raíz explícita (ya existe `with_dir`) y resolver `new_default` por plataforma o dejar `new_default` solo escritorio. |
| `src/app.rs` | Dividir en módulos; conservar acciones; mover UI de escritorio; añadir dispatch por `LayoutMode`. |
| `src/app/mobile.rs` | Nuevo: barra superior, journals, secciones, páginas, editor, compositor, hojas, modales, ajustes. |
| `src/app/widgets.rs` | Nuevo: helpers de fuente, botón táctil, hoja inferior, snackbar, fila de anotación. |
| `src/platform/*` | Nuevo: rutas de datos y manejo del botón atrás. |
| `src/app.rs` (`load_cjk_font`) | `#[cfg(not(target_os = "android"))]` o carga de fuente embebida. |
| `assets/fonts/` | Añadir Noto Sans JP (OFL) si se mantiene CJK en Android. |
| `packaging/android/` | Iconos, manifest/plantilla Gradle, reglas de backup. |
| `.github/workflows/android.yml` | Nuevo workflow de build/APK. |
| `.github/workflows/release.yml` | Adjuntar APK al release. |
| `README.md` | Sección de instalación Android y requisitos. |

---

## 8. Detalles de build Android

### Enfoque elegido — Gradle + `cargo-ndk` + GameActivity

- Proyecto Android mínimo en `android/` con `GameActivity`.
- `cargo install cargo-ndk`.
- Feature `android-game-activity`.
- Ventaja: se mantiene AccessKit/TalkBack y el control del back/predictive back.
- Coste: más configuración, dependencia de Gradle y del AAR de GameActivity.

Variante descartada: `cargo-apk` + NativeActivity. Era más rápida de montar, pero pierde AccessKit en Android y da menos control sobre el ciclo de vida y el botón atrás. No se usará ni como paso intermedio para evitar retrabajo.

### Configuración de app

- Package id: `org.ujournal.UJournal`.
- Label: `UJournal`.
- `versionName` desde `CARGO_PKG_VERSION`; `versionCode` derivado de la versión (por ejemplo, `major*10000 + minor*100 + patch`).
- `minSdk`: 26 inicialmente; confirmar requisito real de winit/eframe.
- `targetSdk`: el más reciente estable soportado por el toolchain.
- Permisos: ninguno (solo almacenamiento interno).
- `android:allowBackup="true"` para conservar journals al cambiar de dispositivo; documentar.
- Icono: mipmaps desde `assets/app-icon.png`; valorar icono adaptativo.
- Orientación: `unspecified`; probar vertical y horizontal.

---

## 9. Datos y compatibilidad

- Ruta Android: `internal_data_path().join("journals")`, equivalente a `/data/user/0/org.ujournal.UJournal/files/journals`.
- Sin permisos de almacenamiento y sin acceso a carpetas externas.
- Mismo formato `.txt` y misma codificación UTF-8 que escritorio.
- Mantener el nombre lógico/directorio actual en escritorio (`UrustJournal`) para no romper instalaciones existentes; en Android usar el nuevo paquete.
- Añadir más adelante “Exportar journal” (compartir archivo `.txt` con el selector de Android) para mover datos sin cables.
- Probar migración manual: copiar un `.txt` de escritorio a la carpeta interna (o importarlo) y abrirlo en Android.

---

## 10. CI y release

Workflow Android:

1. Checkout.
2. Instalar JDK, Android SDK/NDK y targets Rust.
3. Cachear Cargo/Gradle.
4. `cargo test` (host) antes del build Android.
5. Build debug en PR; build release en tags.
6. Firmar con secretos (`ANDROID_KEYSTORE_BASE64`, alias, contraseñas).
7. `apksigner verify` y subida del APK.
8. En tags, adjuntar el APK al GitHub Release existente.

Nombres de artefacto sugeridos:

- `UJournal-<version>-android-arm64-v8a.apk`
- `UJournal-<version>-android-x86_64.apk` (emulador)
- `UJournal-<version>-android-universal.apk`

---

## 11. Pruebas

### Automáticas

- Mantener los 34 tests actuales de dominio, formato, nombres, páginas y almacenamiento.
- Añadir tests de resolución de ruta de almacenamiento con directorios temporales.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`.
- Compilación Android en CI para evitar regresiones.
- Opcional: tests de UI con `egui_kittest` para flujos críticos (crear journal, crear página, añadir bullet).

### Manuales en dispositivo/emulador

- Crear/renombrar/borrar journals; nombres inválidos y duplicados.
- Crear/borrar páginas; muchas páginas (scroll horizontal).
- Añadir/editar/mover/borrar anotaciones; cambiar marcador y color.
- Completar tareas y ver tachado.
- Cambiar los 6 idiomas y comprobar que no se corta el texto.
- Cambiar entre varios de los 34 temas; contraste y legibilidad.
- Rotar, minimizar y volver; comprobar que no se pierden datos.
- Abrir/cerrar el teclado; escribir texto largo y con emojis/CJK.
- Botón atrás en cada nivel (hoja → editor → journals → salir).
- Fuente del sistema grande, modo oscuro y TalkBack (si aplica).
- Uso prolongado: 200+ bullets, apertura rápida, consumo de batería.

---

## 12. Riesgos y mitigaciones

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| IME/teclado tapa el compositor | Alto | Compositor en panel inferior, probar Gboard/SwiftKey, safe areas. |
| Botón atrás cierra la app | Alto | Manejar `Key::BrowserBack` y orden de navegación; wrapper si hace falta. |
| Scroll anidado en el editor | Medio | Un solo ScrollArea en móvil; mover compositor fuera del scroll. |
| `ProjectDirs` inválido en Android | Alto | Usar `internal_data_path()` y fallback con error claro. |
| Fuente CJK no disponible | Medio | Empaquetar Noto Sans JP o desactivar JP en Android temporalmente. |
| Configuración Gradle/GameActivity más compleja | Medio | Proyecto Android mínimo versionado; documentar SDK/NDK y cachear en CI. |
| Contexto OpenGL perdido al reanudar | Medio | Probar ciclo de vida; eframe/glow debería recrear; guardar antes de pausar. |
| APK grande | Bajo/Medio | APK por ABI, `strip`, LTO, limitar fuentes. |
| Pérdida de datos al matar la app | Alto | Guardado al perder foco/pausa + autoguardado actual. |
| Rotación reinicia estado | Medio | Mantener estado en `UJournalApp`; eframe conserva la app; probar. |
| Fragmentación de versiones Android | Medio | API mínima realista y matriz de QA. |

---

## 13. Decisiones pendientes

1. ¿Se exige TalkBack/AccessKit en Android desde la primera release? Si sí, ir a GameActivity + Gradle desde el principio.
2. ¿APK universal, por ABI o ambos?
3. ¿Bloquear orientación a vertical en la primera versión?
4. ¿Versión mínima de Android (API 26 vs. otra)?
5. ¿Dónde se guarda la keystore de release y quién la custodia?
6. ¿Se añade exportación/importación de journals en esta misma iteración?
7. ¿Se mantiene el directorio de datos `UrustJournal` en escritorio y solo se usa el paquete nuevo en Android? (Recomendado: sí.)

---

## 14. Orden de implementación sugerido

1. Fase 0 + Fase 1: APK mínimo que arranca y lista journals.
2. Fase 2: almacenamiento Android, fuentes y guardado al pausar.
3. Refactor de `src/app.rs` en módulos sin cambiar comportamiento de escritorio.
4. Fase 3: UI móvil con paridad funcional.
5. Fase 4: pulido táctil, hojas, compositor y confirmaciones.
6. Fase 5: firma y APK release.
7. Fase 6: CI Android y publicación en tags.
8. Fase 7: QA en emulador y dispositivo real.

Entregables:

- Este plan.
- APK debug funcional en emulador.
- APK release firmado e instalable.
- Workflow de CI que adjunta el APK al release.
- Documentación de instalación y limitaciones en `README.md`.
