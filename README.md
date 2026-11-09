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
- **Multiplataforma:** disponible para Linux, Windows y macOS.

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

## Licencia

Distribuido bajo licencia dual MIT OR Apache-2.0. Consulta `LICENSE-MIT` y `LICENSE-APACHE`.

La fuente [Caveat](https://fonts.google.com/specimen/Caveat) se incluye bajo licencia SIL OFL 1.1 (`assets/fonts/OFL-Caveat.txt`).
