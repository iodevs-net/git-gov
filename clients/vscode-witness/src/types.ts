export type NavigationType = 'scroll' | 'file_switch' | 'go_to_definition' | 'hover';

export type SensorEvent =
  | { type: 'focus_gained'; file_path: string | null; timestamp_ms: number }
  | { type: 'focus_lost'; timestamp_ms: number }
  | { type: 'edit_burst'; file_path: string; chars_delta: number; timestamp_ms: number }
  | { type: 'navigation'; file_path: string; nav_type: NavigationType; timestamp_ms: number }
  | { type: 'heartbeat'; timestamp_ms: number }
  | { type: 'disconnect'; timestamp_ms: number };
