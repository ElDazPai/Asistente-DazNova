# Asistente DazNova

Asistente personal inteligente de escritorio. Vive en segundo plano como un widget flotante,
razona con modelos en la nube o locales, recuerda el contexto y actúa sobre tu equipo con permiso.

- **Shell:** Tauri 2 (ventana nativa, bajo consumo, integración con el SO)
- **Backend:** Rust (procesos, planificador, notificaciones, automatización)
- **UI:** Svelte 5 + TypeScript + CSS propio (tema oscuro, cristal, animaciones ligeras)
- **Datos:** SQLite local
- **IA:** OpenAI API + Ollama (capa única compatible con OpenAI)

## Requisitos

| Herramienta | Versión mínima | Nota |
|---|---|---|
| Node.js | 20 | Probado con 22 |
| Rust (rustup) | 1.77 | Toolchain `x86_64-pc-windows-msvc` |
| Visual Studio Build Tools | 2022 | Workload "Desarrollo para el escritorio con C++" |
| WebView2 | — | Preinstalado en Windows 10/11 |
| Git | — | — |

Linux: `webkit2gtk-4.1`, `libayatana-appindicator3`, `librsvg2` y `build-essential`.

## Puesta en marcha

```bash
npm install
npm run tauri dev
```

Si PowerShell bloquea `npm.ps1`, usa `npm.cmd` o `npx.cmd`.

## Scripts

| Comando | Qué hace |
|---|---|
| `npm run dev` | Solo el frontend (Vite en `:1420`) |
| `npm run tauri dev` | App completa con recarga en caliente |
| `npm run build` | Compila el frontend |
| `npm run tauri build` | Genera el instalador |
| `npm run check` | Verificación de tipos (svelte-check) |

## Estructura

```text
src/                    Frontend Svelte
  lib/components        Componentes reutilizables
  lib/views             Widget, Chat, Search, Dashboard, Settings
  lib/stores           Estado reactivo
  lib/services         Wrappers tipados de invoke()
  lib/styles           Design tokens y estilos globales
src-tauri/src/          Backend Rust (un módulo por dominio)
src-tauri/migrations/   Migraciones SQL versionadas
docs/                   Plan de desarrollo y decisiones (ADRs)
```

## Documentación

- [`docs/PLAN.md`](docs/PLAN.md) — roadmap por etapas, criterios de salida y estado del proyecto.
- [`docs/DECISIONS.md`](docs/DECISIONS.md) — registro de decisiones técnicas (ADR).

## Privacidad

Las claves de API y los tokens de OAuth se guardan en el llavero del sistema operativo
(Windows Credential Manager / Secret Service), nunca en SQLite. La base de datos local
permanece en tu equipo y está excluida del control de versiones.
