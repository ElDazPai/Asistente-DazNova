/** Iconos de trazo (24×24) usados por la interfaz. */
export type IconName =
  | 'chat'
  | 'search'
  | 'mail'
  | 'calendar'
  | 'habits'
  | 'memory'
  | 'settings'
  | 'expand'
  | 'collapse'
  | 'close'
  | 'mic'
  | 'send'
  | 'stop'
  | 'plus';

export const iconPaths: Record<IconName, string> = {
  chat: 'M4 5h16v10H8l-4 4V5z',
  search: 'M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z M16.2 16.2 21 21',
  mail: 'M3 6h18v12H3z M3 7l9 6 9-6',
  calendar: 'M4 6h16v14H4z M4 10h16 M8 3v4 M16 3v4',
  habits: 'M4 12.5l4.5 4.5L20 5.5',
  memory: 'M6 4h12v16H6z M9 8h6 M9 12h6 M9 16h3',
  settings:
    'M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z M12 2v3 M12 19v3 M2 12h3 M19 12h3 M4.9 4.9l2.1 2.1 M17 17l2.1 2.1 M19.1 4.9 17 7 M7 17l-2.1 2.1',
  expand: 'M4 9V4h5 M20 15v5h-5 M4 4l6 6 M20 20l-6-6',
  collapse: 'M9 4v5H4 M15 20v-5h5 M9 9 4 4 M15 15l5 5',
  close: 'M6 6l12 12 M18 6 6 18',
  mic: 'M12 4a3 3 0 0 1 3 3v5a3 3 0 0 1-6 0V7a3 3 0 0 1 3-3z M5 11a7 7 0 0 0 14 0 M12 18v3',
  send: 'M4 12l16-8-6 16-3-6-7-2z',
  stop: 'M8 8h8v8H8z',
  plus: 'M12 5v14 M5 12h14',
};
