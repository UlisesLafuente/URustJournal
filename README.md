# UJournal

UJournal es una aplicación de *bullet journal* multiplataforma, escrita en Rust, pensada para anotar rápido y con una estética cálida tipo cuaderno.

## Características

- **Organización tipo bullet journal:** journals, secciones (Future Log, Monthly Log y Journal) y páginas que creas a tu medida.
- **Anotaciones con significado:** tareas, completadas, migradas, programadas, eventos y notas, con colores para resaltar lo importante.
- **Escritura a mano:** fuente estilo manuscrita para una experiencia más cercana al papel.
- **34 temas:** paletas que van de lo sobrio a lo acogedor (nordic, bosque, lluvia, chimenea, biblioteca, etc.).
- **Multilingüe:** interfaz en español, inglés, alemán, italiano, portugués y japonés, con cambio en tiempo real.
- **Gestión cómoda:** menú contextual en la lista de journals para crear, renombrar o eliminar; mover anotaciones y opciones rápidas sin complicaciones.
- **Interfaz despejada:** paneles plegables y guardado automático mientras trabajas.
- **Accesible:** navegación por teclado y compatibilidad con tecnologías de asistencia.
- **Multiplataforma:** disponible para Linux, Windows, macOS y Android, con una interfaz táctil de una columna en el teléfono.

## Uso

```bash
cargo run
```

Para verificar el proyecto:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Android

El APK usa GameActivity, compila las bibliotecas nativas para `arm64-v8a` y `x86_64`, y guarda los journals en el almacenamiento interno privado de la app.

```bash
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t x86_64 -o android/app/src/main/jniLibs build --release
cd android && ./gradlew assembleRelease
```

El APK se genera en `android/app/build/outputs/apk/release/app-release.apk`. El workflow `Android` valida cada cambio y el workflow `Release` adjunta el APK a las etiquetas `v*`. Las builds de release usan firma de depuración salvo que se configuren `UJOURNAL_KEYSTORE`, `UJOURNAL_KEYSTORE_PASSWORD`, `UJOURNAL_KEY_ALIAS` y `UJOURNAL_KEY_PASSWORD`.

## Licencia

Distribuido bajo licencia dual MIT OR Apache-2.0. Consulta `LICENSE-MIT` y `LICENSE-APACHE`.

La fuente [Caveat](https://fonts.google.com/specimen/Caveat) se incluye bajo licencia SIL OFL 1.1 (`assets/fonts/OFL-Caveat.txt`).

La fuente [Noto Sans JP](https://fonts.google.com/noto/specimen/Noto+Sans+JP) se incluye para Android bajo licencia SIL OFL 1.1 (`assets/fonts/OFL-NotoSansJP.txt`).
