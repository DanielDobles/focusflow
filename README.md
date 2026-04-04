# FocusFlow 🪟⚡

> **Visual editor nativo de Windows para árboles de focus de Hearts of Iron IV**
> 
> Diseñado para modders de **Millennium Dawn**. Edita, valida y exporta archivos de focus tree sin tocar código manualmente.

---

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75+-orange?logo=rust" alt="Rust 1.75+">
  <img src="https://img.shields.io/badge/Platform-Windows-blue?logo=windows" alt="Windows">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT License">
  <img src="https://img.shields.io/badge/UI-egui-purple" alt="egui">
</p>

---

## ✨ ¿Qué es FocusFlow?

FocusFlow es una aplicación **100% nativa de Windows** (sin Electron, sin Tauri, sin WebViews) construida en **Rust + egui** que permite:

- 🔍 **Ver** todos los focuses de un país como lista interactiva o canvas visual
- ✏️ **Crear y editar** focuses con formulario visual — sin tocar archivos `.txt`
- ✅ **Validar** automáticamente: IDs duplicados, prerequisitos inexistentes, costos fuera de rango
- 💾 **Exportar** a formato Paradox válido con un clic (con backup automático)
- 🌐 **Visualizar** conexiones del árbol con nodos de colores por categoría

## 🎯 ¿Para quién?

| Usuario | Beneficio |
|---------|-----------|
| **Modder nuevo** | Crea focuses sin aprender sintaxis Paradox |
| **Modder experimentado** | Valida cambios en segundos, no en horas |
| **Equipo de desarrollo** | Edición visual consistente, sin errores de formato |
| **Reviewer de PRs** | Diff visual claro entre versiones |

## 🚀 Características

### Edición
- ✨ Crear, editar, duplicar y eliminar focuses
- 📝 Formulario con campos: ID, icono, posición, costo, prerequisitos, filtros, completion_reward, ai_will_do
- ⌨️ Atajos de teclado: `Ctrl+S` guardar, `Ctrl+Z` deshacer, `Ctrl+Y` rehacer, `E` editar, `Del` eliminar

### Vistas
- 📋 **List View** — Tabla con búsqueda y filtros por categoría
- 🌐 **Canvas View** — Nodos visuales con conexiones de prerequisitos (flechas azules) y mutually exclusive (líneas rojas), zoom y pan

### Validación
- ❌ IDs duplicados
- ❌ Prerequisitos que no existen
- ❌ Costos fuera de rango (0.1 - 100)
- ❌ Posiciones fuera de bounds
- ⚠️ Advertencias de costos inusuales

### Archivo
- 💾 Save con **backup automático** (`.txt.bak`)
- 🔄 Reload desde archivo original
- 📊 **Diff preview** entre versión original y editada

### Multi-país
- 🇻🇪 Venezuela
- 🇨🇴 Colombia
- 🇧🇷 Brasil
- *(Cualquier archivo `national_focus/*.txt`)*

## 🖥️ Capturas

### Pantalla principal — Lista de focuses
```
┌──────────────────────────────────────────────────────────┐
│  📂 Open  💾 Save  ✨ New  ✏️ Edit  🗑️ Delete  🔍 Val.  │
├────────────┬─────────────────────────────────────────────┤
│ 🪟 FocusFlow│  📌 VEN_reap_the_fruits                     │
│            │  ───────────────────────────────────────    │
│ 📂 Path... │  Icon: flag_venez                            │
│ 🇻🇪 Load   │  Position: (0, 0)     Cost: 3.7 days       │
│ 🇨🇴 Load   │  Category: 🏛️ Political                    │
│ 🇧🇷 Load   │  Filters: FOCUS_FILTER_POLITICAL            │
│            │                                             │
│ 🔍 Search  │  Completion Reward:                         │
│ 📂 All     │  ┌─────────────────────────────────────┐    │
│            │  │ log = "[Root.GetName]: ..."         │    │
│ 🏛️ 146     │  │ add_political_power = 150           │    │
│ 🏭 57      │  └─────────────────────────────────────┘    │
│ ⚔️ 5       │                                             │
│ 🔬 5       │  [✏️ Edit]  [📋 Duplicate]  [🗑️ Delete]    │
│ 📋 119     │                                             │
└────────────┴─────────────────────────────────────────────┘
```

### Canvas View — Visualización del árbol
```
  [VEN_cartel] ────→ [VEN_rise_of_5th]
       │                    │
       ↓                    ↓
  [VEN_we_are_power] → [VEN_industry]
       │                    │
       ↓                    ↓
  [VEN_navy] ← ─ ─ ─ ─ [VEN_airforce]
  
  🔵 = Prerequisite (sólido)
  🔴 = Mutually Exclusive (punteado)
```

## 📦 Instalación

### Requisito previo
**Rust** instalado. Si no lo tienes:
```powershell
winget install Rustlang.Rustup
```

### Build
```powershell
git clone https://github.com/DanielDobles/focusflow.git
cd focusflow
cargo build --release
```

### Ejecutar
```powershell
cargo run --release
```

O usa el batch incluido:
```
build_and_run.bat
```

## ⌨️ Atajos de teclado

| Tecla | Acción |
|-------|--------|
| `Ctrl + S` | Guardar archivo |
| `Ctrl + Z` | Deshacer |
| `Ctrl + Y` | Rehacer |
| `E` | Editar focus seleccionado |
| `Delete` | Eliminar focus seleccionado |
| `Ctrl + D` | Duplicar focus seleccionado |
| `F5` | Recargar archivo |

## 🏗️ Arquitectura

```
focusflow/
├── Cargo.toml              # Dependencias: eframe, egui, serde, anyhow
├── src/
│   ├── main.rs             # Entry point, ventana nativa 1200x800
│   ├── app.rs              # UI completa (~1200 líneas)
│   │   ├── Menu bar        # File, Edit, View, Help
│   │   ├── Left panel      # File load, search, focus list
│   │   ├── Center panel    # List view o Canvas view
│   │   ├── Right panel     # Editor o Validation
│   │   └── Keyboard shortcuts
│   ├── model.rs            # FocusTree, FocusNode, ValidationResult
│   ├── parser.rs           # Parser custom de archivos Paradox HOI4
│   └── writer.rs           # Serializador a formato Paradox + diff
└── build_and_run.bat       # Launcher con checks automáticos
```

## 🔧 Stack técnico

| Componente | Tecnología | Por qué |
|------------|-----------|---------|
| **UI** | egui / eframe | Inmediate mode, nativo, sin WebView |
| **Parser** | Custom (línea-por-línea) | Maneja nesting arbitrario de bloques Paradox |
| **Serialization** | serde + serde_json | Undo/redo con snapshots JSON |
| **Error handling** | anyhow | Errores claros con context |

## 📊 Benchmarks

| Métrica | Resultado |
|---------|-----------|
| **Parse venezuela.txt** | ~72ms (332 focuses, 10,590 líneas) |
| **Round-trip parse → write → re-parse** | ✅ Campos idénticos 100% |
| **Braces balanceados en output** | ✅ 100% |
| **Validación completa** | < 1 segundo |

## 🧪 Testeo

```powershell
cargo test
```

9 tests incluidos:
- Parse unitario (focus simple, prerequisites, archivo real)
- Round-trip completo (parse → write → re-parse)
- Validación de archivo real
- Performance (10 iteraciones)
- Categorías y posiciones
- Detección de diffs
- Casos edge (archivo vacío, garbage input, caracteres especiales)
- Braces balanceados en output del writer

## 🔮 Roadmap

### Fase 2
- [ ] Integración con GitHub API (crear rama, commit, PR)
- [ ] Autocompletado de iconos desde `gfx/`
- [ ] Editor visual con drag-and-drop en canvas
- [ ] Snap to grid (96x130px)

### Fase 3
- [ ] Soporte para más países (Argentina, Chile, México...)
- [ ] Exportar a múltiples formatos
- [ ] Instalador `.msi` con WiX
- [ ] Tema claro/oscuro

## 🤝 Contribuir

1. Fork el repo
2. Crea tu rama (`git checkout -b feature/mi-feature`)
3. Commit cambios (`git commit -m 'feat: descripción'`)
4. Push a la rama (`git push origin feature/mi-feature`)
5. Abre un Pull Request

## 📝 Licencia

MIT — Haz lo que quieras con esto.

## 🙏 Créditos

- **Inspirado por**: [pdx-tools](https://github.com/pdx-tools/pdx-tools) — mismo espíritu frictionless
- **UI**: [egui](https://github.com/emilk/egui) — immediate mode GUI en Rust
- **Parser**: Custom, diseñado específicamente para sintaxis Paradox HOI4
- **Testing**: Archivo `venezuela.txt` del repo [Millennium Dawn](https://github.com/DanielDobles/Millennium-Dawn)

---

<p align="center">
  Hecho con 🦀 y ☕ por Daniel Dobles
</p>
