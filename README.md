# URustJournal

Aplicación de *bullet journal* multiplataforma escrita en Rust con [eframe/egui](https://github.com/emilk/egui).

## Características

- Varios journals con nombres ASCII (`[A-Za-z0-9_]`).
- Tres secciones por journal: Future Log, Monthly Log y Journal.
- Páginas creadas por el usuario dentro de cada sección.
- Marcadores *bullet journal*: tarea (`•`), completada (`x`), migrada (`>`), programada (`<`), evento (`o`) y nota (`-`).
- Guardado en archivos UTF-8 `.txt` en el directorio de datos del sistema (`~/.local/share/urustjournal/journals` en Linux).

## Requisitos

- Rust estable (edición 2024).

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
