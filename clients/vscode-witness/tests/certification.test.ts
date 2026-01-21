import { test, expect } from 'vitest';
import fc from 'fast-check';
import { SensorEvent, NavigationType } from '../src/types';

// Arbitraries
const NavigationTypeArbitrary = fc.constantFrom<NavigationType>('scroll', 'file_switch', 'go_to_definition', 'hover');

const SensorEventArbitrary = fc.oneof(
  fc.record({
    type: fc.constant('focus_gained'),
    file_path: fc.option(fc.string(), { nil: null }),
    timestamp_ms: fc.nat()
  }),
  fc.record({
    type: fc.constant('focus_lost'),
    timestamp_ms: fc.nat()
  }),
  fc.record({
    type: fc.constant('edit_burst'),
    file_path: fc.string(),
    chars_delta: fc.integer(),
    timestamp_ms: fc.nat()
  }),
  fc.record({
    type: fc.constant('navigation'),
    file_path: fc.string(),
    nav_type: NavigationTypeArbitrary,
    timestamp_ms: fc.nat()
  }),
  fc.record({
    type: fc.constant('heartbeat'),
    timestamp_ms: fc.nat()
  }),
  fc.record({
    type: fc.constant('disconnect'),
    timestamp_ms: fc.nat()
  })
);

test('Certification: SensorEvent serialization', () => {
  fc.assert(
    fc.property(SensorEventArbitrary, (event) => {
      // Cast to SensorEvent to ensure TS happy, though fast-check infers well usually
      const typedEvent = event as SensorEvent;

      const json = JSON.stringify(typedEvent);

      // Basic check: it is valid JSON
      const parsed = JSON.parse(json);

      // Parity check: parsed object equals original object
      expect(parsed).toEqual(typedEvent);

      // Schema validation checks
      expect(parsed).toHaveProperty('type');
      expect(parsed).toHaveProperty('timestamp_ms');

      switch (parsed.type) {
        case 'focus_gained':
          expect(parsed).toHaveProperty('file_path');
          break;
        case 'edit_burst':
          expect(parsed).toHaveProperty('file_path');
          expect(parsed).toHaveProperty('chars_delta');
          break;
        case 'navigation':
          expect(parsed).toHaveProperty('file_path');
          expect(parsed).toHaveProperty('nav_type');
          break;
      }
    }),
    { numRuns: 10000 }
  );
});
