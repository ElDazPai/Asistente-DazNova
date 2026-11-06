# Registro de decisiones (ADR)

Una decisión = un bloque. Si se revierte, no se borra: se marca como **Reemplazada** y se añade la nueva.

---

## ADR-001 — Tauri 2 + Rust + Svelte 5 + SQLite

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Contexto.** Se necesita una app de escritorio real, ligera, en segundo plano y con acceso al SO,
sin el peso de Electron.

**Decisión.** Tauri 2 para el shell y la integración nativa, Rust para la lógica de fondo,
Svelte 5 para la UI y SQLite para persistencia.

**Consecuencias.** Instalador de pocos MB y RAM en reposo baja, a cambio de requerir toolchain de
Rust para desarrollar. La UI no puede asumir APIs de navegador (no hay servidor propio).

---

## ADR-002 — Un único cliente de IA compatible con OpenAI

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Contexto.** El requisito es independencia de proveedor, con nube (OpenAI) y local (Ollama).

**Decisión.** Toda la IA pasa por una capa que habla el formato `/v1/chat/completions`.
OpenAI, Ollama, OpenRouter, Groq y LM Studio comparten ese contrato.

**Consecuencias.** Un solo camino de código y pruebas. Se renuncia a capacidades exclusivas de
proveedor (p. ej. herramientas nativas no estándar); si alguna se necesita, se implementa como
extensión opcional del adaptador, nunca en el núcleo.

---

## ADR-003 — SQLite gestionado desde Rust, no desde JavaScript

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Contexto.** El planificador de recordatorios y las notificaciones deben funcionar con la ventana
oculta o cerrada. La lógica en el frontend muere cuando el webview se oculta.

**Decisión.** `rusqlite` (con `bundled`) dentro de Rust, migraciones SQL versionadas en
`src-tauri/migrations/`. El frontend solo llama comandos Tauri.

**Consecuencias.** Un único dueño del esquema, transacciones reales y acceso desde hilos de fondo.
Coste: cada consulta nueva requiere un comando Tauri en lugar de SQL directo desde la UI.

---

## ADR-004 — Secretos en el llavero del SO

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Decisión.** API keys y refresh tokens se guardan con el crate `keyring`
(Windows Credential Manager / Secret Service). SQLite solo almacena una referencia (`token_ref`).

**Consecuencias.** Un `assistant.db` filtrado no expone credenciales. Coste: el llavero puede pedir
desbloqueo en Linux; se maneja con reintento y mensaje claro.

---

## ADR-005 — OAuth 2.0 con loopback + PKCE

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Decisión.** Servidor HTTP efímero en `127.0.0.1` con puerto aleatorio y PKCE (RFC 8252).
Sin `client_secret` embebido en el binario.

**Consecuencias.** Flujo aprobado por Google para apps de escritorio y sin secreto extraíble.
Coste: hay que abrir el navegador del sistema, no se puede hacer login embebido.

---

## ADR-006 — Gmail por polling, no por push

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Contexto.** Las notificaciones push de Gmail exigen un endpoint HTTPS público con Google
Cloud Pub/Sub. Un escritorio no lo tiene sin operar un servidor.

**Decisión.** Polling adaptativo con `historyId`: frecuente con la ventana abierta, espaciado en
segundo plano, y parada total si el usuario desconecta.

**Consecuencias.** Latencia de segundos a minutos para correo nuevo, sin infraestructura propia
ni coste de servidor. Se documenta la limitación en la UI de integraciones.

---

## ADR-007 — Memoria con FTS5 antes que con embeddings

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Decisión.** La recuperación de memoria usa SQLite FTS5 (BM25) combinado con importancia y
recencia. Los vectores se evalúan en la Etapa 8 solo si el recall medido es insuficiente.

**Consecuencias.** Cero coste y cero latencia de embedding, y funciona offline. Coste: peor
recuperación semántica ("coche" vs "automóvil") hasta que se añada la capa vectorial.

---

## ADR-008 — Una ventana con tres estados

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Decisión.** Minimizado (72×72), expandido (420×600) y panel (1100×760) son tamaños de la misma
`WebviewWindow`, con transición animada.

**Consecuencias.** Menos RAM que tres ventanas y estado compartido sin IPC. Coste: el layout debe
ser responsivo a tres anchos muy distintos y el reposicionamiento al expandir requiere cuidado.

---

## ADR-009 — Sin framework de UI ni Tailwind

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Decisión.** CSS propio con design tokens en `src/lib/styles/tokens.css`.

**Consecuencias.** Control total sobre transparencias, blur y animaciones, y menos dependencias.
Coste: los componentes base (botón, panel, input) se escriben a mano.

---

## ADR-010 — Atajo global por defecto `Alt+Shift+Space`

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Contexto.** `Ctrl+Space` choca con IME y `Alt+Space` abre el menú de ventana en Windows.

**Decisión.** `Alt+Shift+Space` por defecto, configurable, con detección de conflicto al registrar.

**Consecuencias.** Sin colisiones conocidas en Windows/Linux. Se avisa al usuario si otro programa
ya lo ocupa.

---

## ADR-011 — TypeScript 5.9 en lugar de 7

**Fecha:** 2026-11-06 · **Estado:** Aceptada

**Contexto.** TypeScript 7 (compilador nativo) ya está publicado, pero `svelte-check` 4.7.8 declara
como peer `typescript: ^5 || ^6`. Instalar TS 7 rompería la verificación de tipos del proyecto.

**Decisión.** Fijar `typescript ~5.9.3` hasta que `svelte-check` (o su sucesor) soporte TS 7.
Vite 8 y `@sveltejs/vite-plugin-svelte` 7 sí se usan en su versión actual.

**Consecuencias.** Se pierde la velocidad del compilador nativo en `npm run check`, que hoy tarda
menos de 2 s. Revisar al actualizar svelte-check.
