import { describe, test, expect } from 'vitest';
import fc from 'fast-check';
import { SensorEvent, NavigationType } from '../src/types';

describe('SensorEvent Certification', () => {
  const timestampArb = fc.integer({ min: 0, max: Number.MAX_SAFE_INTEGER });
  const filePathArb = fc.string();
  const nullableFilePathArb = fc.oneof(fc.constant(null), fc.string());
  const navigationTypeArb = fc.constantFrom<NavigationType>('scroll', 'file_switch', 'go_to_definition', 'hover');

  const focusGainedArb = fc.record({
    type: fc.constant('focus_gained'),
    file_path: nullableFilePathArb,
    timestamp_ms: timestampArb,
  });

  const focusLostArb = fc.record({
    type: fc.constant('focus_lost'),
    timestamp_ms: timestampArb,
  });

  const editBurstArb = fc.record({
    type: fc.constant('edit_burst'),
    file_path: filePathArb,
    chars_delta: fc.integer(),
    timestamp_ms: timestampArb,
  });

  const navigationArb = fc.record({
    type: fc.constant('navigation'),
    file_path: filePathArb,
    nav_type: navigationTypeArb,
    timestamp_ms: timestampArb,
  });

  const heartbeatArb = fc.record({
    type: fc.constant('heartbeat'),
    timestamp_ms: timestampArb,
  });

  const disconnectArb = fc.record({
    type: fc.constant('disconnect'),
    timestamp_ms: timestampArb,
  });

  const sensorEventArb = fc.oneof(
    focusGainedArb,
    focusLostArb,
    editBurstArb,
    navigationArb,
    heartbeatArb,
    disconnectArb
  ) as fc.Arbitrary<SensorEvent>;

  test('should generate valid JSON for any SensorEvent', () => {
    fc.assert(
      fc.property(sensorEventArb, (event) => {
        const json = JSON.stringify(event);
        const parsed = JSON.parse(json);

        expect(parsed).toEqual(event);
        expect(parsed.type).toBeDefined();
        expect(parsed.timestamp_ms).toBeDefined();
        expect(typeof parsed.timestamp_ms).toBe('number');

        if (parsed.type === 'focus_gained') {
          expect(parsed).toHaveProperty('file_path');
        } else if (parsed.type === 'edit_burst') {
          expect(parsed).toHaveProperty('file_path');
          expect(parsed).toHaveProperty('chars_delta');
        } else if (parsed.type === 'navigation') {
          expect(parsed).toHaveProperty('file_path');
          expect(parsed).toHaveProperty('nav_type');
        }
      }),
      { numRuns: 10000 } // Mandatory 10,000 runs as per roadmap
    );
  });
});
