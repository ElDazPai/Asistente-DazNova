# Plan de Desarrollo — Asistente-DazNova

> Asistente personal de escritorio (Tauri 2 + Rust + Svelte 5 + SQLite).
> Este documento es la fuente de verdad del proyecto: se actualiza al cerrar cada etapa.

---

## 0. Cómo leer este plan

El proyecto se divide en **9 etapas (0 a 8)**. Cada etapa tiene:

| Campo | Significado |
|---|---|
| **Objetivo** | Qué problema resuelve la etapa en una frase. |
| **Entregables** | Artefactos tangibles que quedan en el repo. |
| **Tareas** | Trabajo concreto, ordenado por dependencia. |
| **Criterio de salida** | Checklist verificable. Si algo falla, la etapa no está cerrada. |
| **🔑 Llave** | La única cosa que hay que tener resuelta para que la siguiente etapa no se caiga. |

Regla de oro: **no se abre una etapa sin haber cerrado la anterior**. Si aparece un bloqueo, se resuelve dentro de la etapa actual, no se arrastra.

---

## 1. Decisiones técnicas base

### 1.1 Stack confirmado

| Capa | Tecnología | Versión objetivo |
|---|---|---|
| Shell de escritorio | Tauri 2 | 2.x |
| Backend nativo | Rust (edition 2021) | stable |
| UI | Svelte 5 (runes) + TypeScript + Vite | 5.x |
| Estilos | CSS puro con design tokens (custom properties) | — |
| Base de datos | SQLite vía `rusqlite` + migraciones propias | 3.4x |
| IA nube | OpenAI API | — |
| IA local | Ollama | — |
| Búsqueda web | Brave Search API (principal), Tavily (alternativa) | — |
| Google | Gmail API + Calendar API, OAuth 2.0 + PKCE | — |

### 1.2 Decisiones que fijo yo (y su justificación)

Estas decisiones evitan rehacer trabajo más adelante. Si alguna no te convence, se cambia **ahora**, no en la etapa 5.

1. **Un solo cliente de IA compatible con OpenAI.**
   Todo el acceso a modelos pasa por una capa que habla el formato `/v1/chat/completions`. Eso cubre OpenAI, Ollama, OpenRouter, Groq y LM Studio sin código extra. La "independencia de proveedor" del requisito se implementa aquí, no con abstracciones por proveedor.

2. **SQLite se maneja desde Rust (`rusqlite`), no desde JavaScript.**
   El planificador de recordatorios y las notificaciones deben funcionar con la ventana oculta o cerrada. Si la lógica vive en el frontend, muere cuando el webview se oculta. El frontend solo consume comandos Tauri.

3. **Los secretos viven en el keyring del sistema operativo, nunca en SQLite.**
   API keys y refresh tokens → Windows Credential Manager / Secret Service (crate `keyring`). En SQLite solo se guarda una *referencia* (`token_ref`). Un `assistant.db` filtrado no debe contener credenciales.

4. **OAuth con loopback + PKCE (RFC 8252).**
   Servidor HTTP efímero en `127.0.0.1` con puerto aleatorio, sin `client_secret` embebido. Es el flujo correcto para apps de escritorio y el único que Google sigue aprobando.

5. **Gmail sin notificaciones push: polling inteligente.**
   El push de Gmail exige un endpoint HTTPS público con Pub/Sub; un escritorio no lo tiene sin montar un servidor. Se usa polling adaptativo (frecuente con la ventana abierta, espaciado en segundo plano) + `historyId` para traer solo lo nuevo. Documentar esta limitación desde ya.

6. **Búsqueda en memoria con FTS5 antes que con embeddings.**
   SQLite FTS5 da búsqueda por relevancia con cero coste y cero latencia. Los embeddings (vectores) se añaden en la etapa 8 solo si el recall medido no alcanza. Evita pagar tokens de embedding desde el día uno.

7. **Una sola ventana con tres tamaños, no tres ventanas.**
   Minimizado / expandido / panel completo son estados de la misma `WebviewWindow` (72×72 → 420×600 → 1100×760). Menos RAM, transiciones instantáneas y estado compartido sin IPC extra.

8. **Sin framework de UI ni Tailwind.**
   Design tokens en `src/styles/tokens.css` + componentes propios. El requisito es un look futurista con cristal y transparencias; un framework genérico pelearía contra eso y sumaría peso.

9. **Atajo global por defecto: `Alt+Shift+Space`.**
   `Ctrl+Space` choca con IME y `Alt+Space` abre el menú de ventana de Windows. Configurable en ajustes.

### 1.3 Riesgos conocidos y mitigación

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Transparencias + ventana sin bordes se ven mal en Windows (bordes negros, sombras raras) | Alto (es la estética del producto) | Validar en Etapa 1 con `decorations:false`, `transparent:true`, `shadow:false`; fallback a fondo opaco con blur simulado |
| El atajo global roba el foco y molesta al escribir | Alto (mata la utilidad) | Etapa 1: `set_focus()` solo si el usuario invoca; probar "no robar foco" como criterio de salida |
| Coste descontrolado de la API de IA | Medio | Registro de tokens y coste por mensaje desde la Etapa 2; límite diario configurable |
| OAuth de Google rechaza la app por scopes excesivos | Alto (bloquea Etapa 5) | Pedir scopes incrementales (`gmail.readonly` primero, `gmail.send` solo al activar envío); modo "app en pruebas" con cuenta propia |
| Recordatorios que no suenan en segundo plano | Alto (rompe la promesa del producto) | Planificador en Rust con `tokio` + notificaciones nativas; prueba explícita con ventana oculta |
| Ollama no instalado en la máquina del usuario | Bajo | Detección en ajustes; el asistente funciona 100% en la nube sin él |
| Autostart y transparencias se comportan distinto en Linux | Medio | Etapa 1 valida ambos SO; documentar diferencias (`.desktop` vs registro) |

---

## 2. Arquitectura de carpetas objetivo

```text
Asistente-DazNova/
├── docs/
│   ├── PLAN.md                 # este documento
│   └── DECISIONS.md            # ADRs cortos (una decisión = un párrafo)
│
├── src/                        # frontend Svelte
│   ├── lib/
│   │   ├── components/         # Button, GlassPanel, Avatar, MessageBubble...
│   │   ├── views/              # Widget, Chat, Search, Dashboard, Settings
│   │   ├── stores/             # estado reactivo (runes)
│   │   ├── services/           # wrappers tipados de invoke()
│   │   └── styles/             # tokens.css, global.css
│   ├── App.svelte
│   └── main.ts
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs              # builder, plugins, estado global
│   │   ├── db/                 # conexión, migraciones, repositorios
│   │   ├── ai_engine/          # cliente OpenAI-compatible, prompts, tools
│   │   ├── memory/             # hechos, preferencias, búsqueda FTS5
│   │   ├── web_search/         # Brave / Tavily, extracción y resumen
│   │   ├── gmail/              # OAuth, listado, resumen, borradores
│   │   ├── calendar/           # eventos, disponibilidad
│   │   ├── scheduler/          # recordatorios, hábitos, tareas periódicas
│   │   ├── notifications/      # notificaciones nativas + bandeja
│   │   ├── system/             # archivos, procesos, autostart, atajos
│   │   ├── secrets/            # keyring
│   │   └── window/             # estados, posición, always-on-top
│   ├── migrations/             # 0001_init.sql, 0002_...
│   ├── icons/
│   ├── capabilities/           # permisos Tauri 2 por ventana
│   └── tauri.conf.json
│
├── database/                   # assistant.db en desarrollo (ignorado por git)
├── package.json
└── README.md
```

---

## 3. Modelo de datos v1

Se crea completo en la Etapa 3, pero se define aquí para que las etapas 2, 4 y 5 no inventen tablas a medias.

| Tabla | Propósito |
|---|---|
| `settings` | `key` / `value` JSON: preferencias, proveedor activo, límites |
| `conversations` | Hilos de chat (`id`, `title`, `archived`, timestamps) |
| `messages` | Mensajes con `role`, `content`, `model`, `tokens_in/out`, `tool_calls` JSON |
| `memories` | Hechos y preferencias: `kind` (`preference`/`fact`/`project`/`person`), `content`, `importance`, `last_used_at` |
| `memories_fts` | Índice FTS5 sobre `memories.content` |
| `tasks` | Recordatorios y tareas: `due_at`, `recurrence` (RRULE), `status`, `notified_at` |
| `habits` | Hábitos: `name`, `schedule`, `target`, `active` |
| `habit_logs` | Registro diario de cumplimiento |
| `integrations` | `provider`, `account_email`, `scopes`, `token_ref`, `expires_at`, `status` |
| `emails_cache` | Metadatos + resumen de correos ya procesados |
| `events_cache` | Eventos de calendario sincronizados |
| `search_history` | Consultas, proveedor, resultados y resumen |
| `tool_audit` | Toda ejecución de herramienta: args, decisión del usuario, resultado |
| `usage_log` | Tokens y coste por llamada de IA |
| `schema_migrations` | Control de versiones del esquema |

---

## 4. Roadmap por etapas

---

### Etapa 0 — Cimientos y decisiones

**Objetivo:** que el repositorio exista, compile y arranque en blanco, con las decisiones cerradas.

**Entregables**
- Repo git inicializado con `.gitignore` correcto (excluye `node_modules`, `target`, `*.db`, `.env`).
- Proyecto Tauri 2 + Svelte 5 + TypeScript funcionando (`npm run tauri dev` abre ventana).
- `docs/PLAN.md` y `docs/DECISIONS.md` (primer ADR con las decisiones de §1.2).
- `README.md` con requisitos previos y comandos.
- Verificación de toolchain: Node, npm, Rust, cargo, WebView2, compilador de Windows.

**Tareas**
1. Comprobar toolchain y versiones; documentar lo que falte instalar.
2. Scaffold Tauri 2 (`create-tauri-app`) con plantilla Svelte-TS.
3. Limpiar la plantilla: estructura de `src/` y `src-tauri/src/` según §2.
4. Configurar `tauri.conf.json`: identificador, título, tamaño inicial, CSP mínima.
5. `.gitignore`, `rustfmt.toml`, `eslint`/`prettier` (opcional pero recomendado).
6. Primer commit: "chore: scaffold Tauri 2 + Svelte 5".

**Criterio de salida**
- [x] `npm run tauri dev` abre una ventana sin errores en consola.
- [x] `cargo build` en `src-tauri/` termina sin warnings bloqueantes.
- [x] `npm run build` genera el bundle del frontend.
- [x] El repo no contiene secretos ni la base de datos.

**Verificado el 2026-11-06:** ventana "Asistente DazNova" abierta, 24,9 MB de RAM en reposo,
`cargo check` y `cargo build` en verde, `svelte-check` 0 errores, bundle de 36 kB.
Toolchain instalada: Rust 1.99.1 (MSVC) + Build Tools 17.14.

**🔑 Llave:** *el entorno compila y arranca en limpio*. Sin esto verde, cada etapa posterior se convierte en depuración de infraestructura.

**Estimación:** 1 sesión.

---

### Etapa 1 — Shell de escritorio (el widget)

**Objetivo:** tener la "presencia" del producto: un widget flotante que aparece, se expande y desaparece sin molestar.

**Entregables**
- Ventana sin bordes, transparente, always-on-top, fuera de la barra de tareas en modo widget.
- Tres estados con transición animada: minimizado (72×72), expandido (420×600), panel completo (1100×760).
- Arrastrable y con posición persistida entre reinicios.
- Atajo global `Alt+Shift+Space` que alterna mostrar/ocultar.
- Indicador de notificaciones en el estado minimizado.
- Icono en bandeja del sistema con menú (Mostrar / Ocultar / Ajustes / Salir).
- Autostart configurable (activado por defecto).

**Tareas**
1. Configurar la ventana (`decorations:false`, `transparent:true`, `skipTaskbar:true`, `alwaysOnTop`).
2. Implementar comandos Rust: `set_window_state(state)`, `save_position()`, `restore_position()`.
3. Zona de arrastre (`data-tauri-drag-region`) y `startDragging()` en el widget.
4. Plugin `global-shortcut` con registro/desregistro y manejo de conflictos.
5. Plugin de bandeja (`tray-icon`) con menú nativo.
6. Plugin `autostart` + toggle en ajustes.
7. Tokens de diseño: colores, radios, blur, sombras, tipografía; `GlassPanel` base.
8. Animaciones de transición (Svelte `transition:` / CSS) con duración ≤180 ms.
9. Validar en Windows y Linux; documentar diferencias.

**Criterio de salida**
- [ ] El atajo abre y cierra el widget en <150 ms percibidos.
- [ ] Al abrirse, no se pierde el texto que el usuario estaba escribiendo en otra app (prueba real en el navegador/editor).
- [ ] La posición se mantiene tras reiniciar la app.
- [ ] Los tres estados se ven correctamente con transparencias (sin halos negros).
- [ ] Autostart funciona tras reiniciar el sistema.

**🔑 Llave:** *el widget no debe robar el foco ni estorbar*. Si invocarlo interrumpe lo que estabas haciendo, el producto fracasa aunque el resto funcione.

**Estimación:** 2–3 sesiones.

---

### Etapa 2 — Núcleo de IA y chat

**Objetivo:** conversación real con streaming, historial persistente y control de coste.

**Entregables**
- Cliente de IA en Rust (`ai_engine`) compatible con OpenAI y Ollama.
- Streaming de respuestas token a token hacia el frontend vía eventos Tauri.
- Cancelación de una respuesta en curso sin corromper el historial.
- Historial de conversaciones en SQLite con títulos autogenerados.
- Registro de tokens y coste por mensaje en `usage_log`.
- Ajustes: proveedor, modelo, temperatura, límite diario de gasto, API key (guardada en keyring).
- Prompt de sistema base con la identidad del asistente y la fecha actual.

**Tareas**
1. Esquema mínimo de BD (`settings`, `conversations`, `messages`, `usage_log`) + migración `0001`.
2. Capa `db/` con repositorios y pool de conexiones.
3. Trait `AiProvider` + implementación OpenAI-compatible (HTTP con `reqwest`, SSE).
4. Comando `send_message` con `Channel`/eventos para streaming; comando `cancel_generation`.
5. UI de chat: burbujas, markdown seguro, estados (pensando / streaming / cancelado / error).
6. Gestión de errores: sin red, 401, rate limit, contexto excedido → mensajes claros.
7. Guardado de API key en keyring + validación al guardar.
8. Auto-título de conversación con un modelo barato.

**Criterio de salida**
- [ ] Conversación fluida con streaming visible (<300 ms hasta el primer token en condiciones normales).
- [ ] Cancelar a mitad deja el historial coherente y permite seguir conversando.
- [ ] Cerrar y reabrir la app conserva las conversaciones.
- [ ] Cada mensaje registra tokens y coste; el total del día se ve en ajustes.
- [ ] Cambiar de OpenAI a Ollama no requiere tocar código, solo ajustes.

**🔑 Llave:** *el streaming debe poder cancelarse y el coste debe medirse desde el primer mensaje*. Un chat sin cancelación ni telemetría de gasto es un chat que no puedes dejar abierto todo el día.

**Estimación:** 3 sesiones.

---

### Etapa 3 — Memoria persistente

**Objetivo:** que el asistente recuerde sin que se lo pidan.

**Entregables**
- Esquema completo de §3 con migraciones versionadas.
- Extracción automática de hechos/preferencias al cerrar una conversación (modelo barato + validación).
- Búsqueda de memoria con FTS5 e inyección de los recuerdos relevantes en el contexto.
- Pantalla de memoria: ver, editar, borrar, marcar importancia.
- Registro de `last_used_at` para saber qué memoria sirve y cuál es ruido.

**Tareas**
1. Migraciones `0002_memory.sql` con `memories` y `memories_fts`.
2. Servicio `memory`: CRUD, búsqueda, ranking (relevancia FTS × importancia × recencia).
3. Pipeline de extracción post-conversación con deduplicación (no guardar dos veces lo mismo).
4. Ensamblado de contexto: sistema + memorias relevantes + últimos N mensajes + resumen de lo antiguo.
5. Compactación de conversaciones largas (resumen + ventana deslizante).
6. UI de memoria en el panel completo.
7. Tests unitarios del ranking y de la deduplicación.

**Criterio de salida**
- [ ] Decir "recuerda que prefiero respuestas cortas" afecta a las respuestas de la sesión siguiente.
- [ ] Preguntar por un proyecto mencionado hace días devuelve la información correcta.
- [ ] Una memoria borrada no reaparece en el contexto.
- [ ] El contexto enviado al modelo nunca excede el presupuesto de tokens definido.

**🔑 Llave:** *la memoria vale por el recall, no por el volumen*. Mide cuántas veces el asistente usa un recuerdo sin que se lo pidas; si no lo usa, guardar más datos solo encarece cada mensaje.

**Estimación:** 3–4 sesiones.

---

### Etapa 4 — Buscador web

**Objetivo:** responder con información actual y con fuentes verificables.

**Entregables**
- Vista de búsqueda dentro del widget con resultados, resumen y enlaces.
- Integración con Brave Search API (y adaptador Tavily).
- Extracción del contenido de las páginas top (legibilidad + límite de tamaño).
- Resumen con citas numeradas que enlazan a la fuente original.
- Historial de búsquedas reutilizable.
- Herramienta `web_search` disponible para el motor de IA (base del tool calling de la Etapa 7).

**Tareas**
1. Módulo `web_search`: cliente Brave, normalización de resultados, manejo de cuotas.
2. Fetcher de páginas con timeout, límite de bytes, respeto de `robots.txt` y user-agent identificable.
3. Extractor de contenido principal (crate de readability) → texto limpio.
4. Resumen con citas: prompt que obliga a referenciar `[1]`, `[2]` y validación de que las citas existen.
5. UI de resultados con tarjetas, favicon, dominio y tiempo de lectura.
6. Cache de resultados por consulta (TTL corto) para no gastar cuota.
7. Historial en `search_history`.

**Criterio de salida**
- [ ] Una pregunta de actualidad devuelve respuesta con al menos 2 fuentes clicables.
- [ ] Toda afirmación del resumen tiene cita; el validador rechaza respuestas sin fuente.
- [ ] Sin API key, la vista lo explica y ofrece el modo sin búsqueda (no rompe la app).
- [ ] Una consulta repetida en <10 min no consume cuota.

**🔑 Llave:** *sin citas clicables, el buscador es una fábrica de alucinaciones*. La fuente no es un adorno: es el mecanismo de verificación del usuario.

**Estimación:** 3 sesiones.

---

### Etapa 5 — Google: Gmail y Calendar

**Objetivo:** conectar la vida digital del usuario de forma segura y duradera.

**Entregables**
- OAuth 2.0 con PKCE y loopback, tokens en keyring, refresh silencioso.
- Panel de Gmail: últimos correos, resumen de hilos, detección de urgencia, búsqueda en lenguaje natural.
- Borradores de respuesta generados y envío **solo con confirmación explícita**.
- Panel de Calendar: próximos eventos, disponibilidad, creación de eventos.
- Polling adaptativo con `historyId` (sin push, ver §1.2.5).
- Página de integraciones: conectar, estado, scopes concedidos, desconectar y borrar datos.

**Tareas**
1. Módulo `secrets` (keyring) + estructura `integrations`.
2. Flujo OAuth: servidor loopback efímero, PKCE, intercambio de código, almacenamiento de refresh token.
3. Cliente Gmail: `users.messages.list/get`, parseo MIME, limpieza de HTML.
4. Clasificación y urgencia con modelo barato + reglas (remitente conocido, plazos, palabras clave).
5. Resumen de hilos largos con presupuesto de tokens.
6. Borradores (`drafts.create`) y envío (`messages.send`) con diálogo de confirmación que muestre destinatario, asunto y cuerpo.
7. Cliente Calendar: `events.list`, `freebusy.query`, `events.insert`.
8. Sincronización y cache; botón de "desconectar" que revoca y borra.
9. Manejo de token revocado / expirado con re-autenticación guiada.

**Criterio de salida**
- [ ] Conectar la cuenta funciona y sobrevive a reinicios sin re-login (refresh silencioso).
- [ ] Desconectar elimina tokens del keyring y los datos cacheados.
- [ ] Un envío de correo exige confirmación y queda en `tool_audit`.
- [ ] Los scopes pedidos son los mínimos necesarios y se ven en la UI.
- [ ] Sin conexión a Google, el resto de la app sigue funcionando.

**🔑 Llave:** *si el usuario tiene que volver a iniciar sesión cada vez, la integración está rota*. El refresh silencioso y la revocación limpia son la línea entre "integración" y "demo".

**Estimación:** 4–5 sesiones.

---

### Etapa 6 — Recordatorios, hábitos y notificaciones

**Objetivo:** convertir el asistente en acompañante que actúa aunque no lo mires.

**Entregables**
- CRUD de recordatorios con recurrencia (RRULE), prioridad y snooze.
- Hábitos con registro diario y estadísticas de cumplimiento.
- Planificador en Rust que dispara notificaciones nativas con la ventana oculta.
- Bandeja de notificaciones + badge en el widget minimizado.
- Lenguaje natural → recordatorio ("recuérdame beber agua cada 2 horas").
- Pausas activas y sugerencias proactivas configurables.

**Tareas**
1. Migraciones `tasks`, `habits`, `habit_logs`.
2. Motor de recurrencia (crate `rrule`) y cálculo de próximas ocurrencias.
3. Planificador `tokio` con tick de 30 s, consulta de vencidos y `notified_at` para no duplicar.
4. Plugin `notification` + acción "abrir asistente" al pulsar.
5. Snooze (5/15/60 min) y "marcar hecho" desde la notificación si el SO lo permite.
6. Parseo de lenguaje natural a tarea (fecha/hora/recurrencia) con confirmación previa.
7. UI de hábitos con heatmap y racha.
8. Modo "no molestar" por franjas horarias.

**Criterio de salida**
- [ ] Un recordatorio suena con la ventana oculta y con la app minimizada a bandeja.
- [ ] No se duplican notificaciones al reiniciar la app.
- [ ] "Recuérdame X mañana a las 9" crea la tarea correcta tras confirmar.
- [ ] El heatmap de hábitos refleja el registro real.
- [ ] El modo no molestar suprime notificaciones sin perderlas (quedan en bandeja).

**🔑 Llave:** *un recordatorio que no suena con la app en segundo plano no existe*. Todo lo demás de esta etapa es interfaz alrededor de esa única garantía.

**Estimación:** 3–4 sesiones.

---

### Etapa 7 — Herramientas, permisos y automatización

**Objetivo:** que el asistente actúe en el equipo con control y trazabilidad.

**Entregables**
- Sistema de tool calling: el modelo decide qué herramienta usar y el backend la ejecuta.
- Herramientas v1: `web_search`, `read_file`, `write_file` (con confirmación), `list_dir`, `open_app`, `create_reminder`, `search_memory`, `send_email` (confirmación).
- Sistema de permisos por herramienta (`allow` / `ask` / `deny`) editable en ajustes.
- Diálogo de confirmación con vista previa de la acción (ruta, comando, destinatario).
- Auditoría completa en `tool_audit` con posibilidad de exportar.
- Sandbox de rutas: lista blanca de directorios accesibles.

**Tareas**
1. Definición de herramientas en JSON Schema y registro en Rust.
2. Bucle de agente: petición → tool_call → resultado → respuesta, con límite de iteraciones.
3. Evaluador de permisos previo a la ejecución.
4. Diálogo de confirmación asíncrono (el backend espera la decisión del usuario).
5. Validación de rutas (canonicalización, prevención de traversal) y de comandos.
6. Auditoría + exportación.
7. Tests de seguridad: intentos de salir del sandbox deben fallar.

**Criterio de salida**
- [ ] "Busca el informe de marzo y resúmelo" funciona de punta a punta.
- [ ] Toda acción destructiva pide confirmación y queda auditada.
- [ ] Una herramienta en `deny` no se ejecuta aunque el modelo la pida.
- [ ] Un intento de escribir fuera de las rutas permitidas es bloqueado y registrado.
- [ ] El bucle de agente no puede ejecutar más de N herramientas por turno.

**🔑 Llave:** *ninguna acción destructiva sin confirmación explícita y sin registro*. La confianza en un agente se construye con un `tool_audit` completo, no con buenas intenciones.

**Estimación:** 4–5 sesiones.

---

### Etapa 8 — Voz, modelos locales y entrega

**Objetivo:** cerrar la visión: manos libres, IA local opcional y app instalable.

**Entregables**
- Entrada por voz (STT) y respuesta hablada (TTS) con activación configurable.
- Detección y uso de Ollama local; selector de modelo por tarea (barato/rápido vs potente).
- Empaquetado e instalador (MSI/NSIS en Windows, AppImage/deb en Linux).
- Autoactualización (`updater`) y firma de artefactos.
- Rendimiento medido: RAM en reposo, CPU en reposo, tiempo de arranque.
- README final, guía de usuario y checklist de privacidad.

**Tareas**
1. STT: motor local (whisper.cpp) o API según ajuste; push-to-talk con el atajo.
2. TTS con voz del sistema o proveedor; interrupción al hablar.
3. Detección de Ollama (`/api/tags`), descarga guiada de modelos, fallback a nube.
4. Perfiles de modelo: `fast` (clasificar, títulos), `smart` (razonamiento), `local` (privacidad).
5. Build de producción firmado; instalador y updater.
6. Perfilado: RAM/CPU en reposo, arranque, consumo por conversación.
7. Documentación final y limpieza de código muerto.

**Criterio de salida**
- [ ] Instalación limpia en una máquina sin Node ni Rust funciona.
- [ ] RAM en reposo < 120 MB y CPU ≈ 0% con el widget oculto.
- [ ] Arranque en frío < 2 s.
- [ ] Una conversación completa funciona 100% offline con Ollama.
- [ ] El updater detecta y aplica una versión nueva.

**🔑 Llave:** *la app debe instalarse, arrancar sola y actualizarse sin intervención*. Hasta que eso no ocurra, es un proyecto; después, es un producto.

**Estimación:** 5–6 sesiones.

---

## 5. Tablero de progreso

| Etapa | Estado | Cerrada el | Notas |
|---|---|---|---|
| 0 — Cimientos | ✅ Cerrada | 2026-11-06 | Tauri 2.12.4 · Svelte 5.57 · Vite 8.3 · TS 5.9 |
| 1 — Shell / widget | ⬜ Pendiente | — | — |
| 2 — Núcleo IA + chat | ⬜ Pendiente | — | — |
| 3 — Memoria | ⬜ Pendiente | — | — |
| 4 — Buscador web | ⬜ Pendiente | — | — |
| 5 — Gmail + Calendar | ⬜ Pendiente | — | — |
| 6 — Recordatorios + hábitos | ⬜ Pendiente | — | — |
| 7 — Herramientas + permisos | ⬜ Pendiente | — | — |
| 8 — Voz + entrega | ⬜ Pendiente | — | — |

Leyenda: ⬜ pendiente · 🟡 en curso · ✅ cerrada · ⛔ bloqueada

---

## 6. Convenciones de trabajo

- **Un commit por tarea cerrada**, mensajes en `tipo(alcance): descripción` (`feat`, `fix`, `refactor`, `docs`, `chore`).
- **Cada etapa termina con**: criterio de salida verificado + tabla de progreso actualizada + `DECISIONS.md` con lo que cambió.
- **Nada de secretos en el repo**: `.env` ignorado, claves en keyring, `*.db` ignorado.
- **Rust**: `cargo fmt` y `cargo clippy` antes de cerrar etapa.
- **Frontend**: `npm run check` (svelte-check) sin errores de tipos.
- **Sin dependencias nuevas** que no estén justificadas en `DECISIONS.md`.
- **Rama por etapa** si el trabajo es grande; `main` siempre compilable.

---

## 7. Presupuesto de recursos (objetivo de producto)

| Métrica | Objetivo | Se mide en |
|---|---|---|
| RAM en reposo (widget oculto) | < 120 MB | Etapa 8 |
| CPU en reposo | ≈ 0% | Etapa 8 |
| Arranque en frío | < 2 s | Etapa 8 |
| Primer token de respuesta | < 1 s (nube) | Etapa 2 |
| Tamaño del instalador | < 15 MB (sin modelos locales) | Etapa 8 |
| Coste de IA por día de uso normal | < 0,30 USD | Etapas 2–8 |
