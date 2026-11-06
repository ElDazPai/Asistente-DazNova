import { invoke } from '@tauri-apps/api/core';

/** Los tres tamaños de ventana que gestiona el backend. */
export type WindowState = 'widget' | 'expanded' | 'dashboard';

export function getWindowState(): Promise<WindowState> {
  return invoke<WindowState>('get_window_state');
}

export function setWindowState(state: WindowState): Promise<void> {
  return invoke<void>('set_window_state', { state });
}

export function toggleWindow(): Promise<void> {
  return invoke<void>('toggle_window');
}

export function hideWindow(): Promise<void> {
  return invoke<void>('hide_window');
}

/** Arranca el arrastre nativo. Debe llamarse tras un movimiento real del puntero. */
export function startDrag(): Promise<void> {
  return invoke<void>('start_drag');
}

export function isAutostartEnabled(): Promise<boolean> {
  return invoke<boolean>('is_autostart_enabled');
}

export function setAutostart(enabled: boolean): Promise<void> {
  return invoke<void>('set_autostart', { enabled });
}
