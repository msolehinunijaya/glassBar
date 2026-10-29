import { describe, expect, it } from 'vitest';
import { connectedText } from './tileText';

describe('what a Wi-Fi / Bluetooth tile says it is connected to', () => {
  it('one name', () => { expect(connectedText(['Unijaya_5G'], 'Not connected')).toBe('Unijaya_5G'); });
  it('the first name and how many others', () => {
    expect(connectedText(['Galaxy Buds', 'MX Keys'], 'No devices')).toBe('Galaxy Buds +1 other');
    expect(connectedText(['Galaxy Buds', 'MX Keys', 'Mouse'], 'No devices')).toBe('Galaxy Buds +2 others');
  });
  it('nothing connected says so', () => { expect(connectedText([], 'No devices connected')).toBe('No devices connected'); });
});
describe('one device seen twice (classic + LE Bluetooth)', () => {
  it('counts once', () => { expect(connectedText(["Sol's Buds3", "Sol's Buds3", 'RK-KB5.0'], 'No devices')).toBe("Sol's Buds3 +1 other"); });
});
