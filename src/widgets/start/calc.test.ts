import { describe, expect, it } from 'vitest';
import { evaluate } from './calc';
import { convert, suggestions, unitOf } from './units';
import { formatNumber, parse } from './query';

describe('calculator', () => {
  it('does arithmetic with precedence and brackets', () => {
    expect(evaluate('2+3*4')).toBe(14);
    expect(evaluate('(2+3)*4')).toBe(20);
    expect(evaluate('2^10')).toBe(1024);
    expect(evaluate('2^3^2')).toBe(512);
    expect(evaluate('-3+5')).toBe(2);
    expect(evaluate('10 / 4')).toBe(2.5);
    expect(evaluate('7 mod 3')).toBe(1);
    expect(evaluate('2(3+1)')).toBe(8);
  });
  it('reads people-style input', () => {
    expect(evaluate('1,000 x 3')).toBe(3000);
    expect(evaluate('6 × 7 ÷ 2')).toBe(21);
    expect(evaluate('15% of 200')).toBe(30);
    expect(evaluate('200 + 10%')).toBe(220);
    expect(evaluate('200 - 10%')).toBe(180);
    expect(evaluate('50%')).toBe(0.5);
    expect(evaluate('15% of 2,400 + 18')).toBe(378);
  });
  it('has functions and constants', () => {
    expect(evaluate('sqrt(16)')).toBe(4);
    expect(evaluate('sin(30)')).toBeCloseTo(0.5);
    expect(evaluate('round(2.567, 2)')).toBe(2.57);
    expect(evaluate('2pi')).toBeCloseTo(6.2832, 3);
    expect(evaluate('abs(-4) + ln(e)')).toBe(5);
  });
  it('says nothing for half-typed or non-maths input', () => {
    for (const q of ['12 +', '(3', '', '5', 'alert(1)', 'hello', 'x', '1/0', 'sqrt(-1)', 'constructor']) expect(evaluate(q), q).toBe(null);
  });
});

describe('units', () => {
  it('converts within a kind', () => {
    expect(convert(5, 'km', 'mi')).toBeCloseTo(3.10686, 4);
    expect(convert(100, 'f', 'c')).toBeCloseTo(37.7778, 3);
    expect(convert(0, 'c', 'k')).toBeCloseTo(273.15);
    expect(convert(1.5, 'gb', 'mb')).toBe(1500);
    expect(convert(90, 'min', 'h')).toBe(1.5);
    expect(convert(70, 'kg', 'lb')).toBeCloseTo(154.324, 2);
  });
  it('refuses across kinds and knows names', () => {
    expect(convert(1, 'kg', 'm')).toBe(null);
    expect(unitOf('Kilometres')?.key).toBe('km');
    expect(unitOf('°F')?.key).toBe('f');
    expect(suggestions(5, 'km').map((s) => s.to)).toEqual(['mi', 'm', 'ft']);
  });
});

describe('search query', () => {
  it('spots maths, units and money', () => {
    expect(parse('2+2')).toEqual({ kind: 'math', value: 4 });
    expect(parse('5 km to mi')).toMatchObject({ kind: 'unit', from: 'km', to: 'mi' });
    expect(parse('70 kg in lb')).toMatchObject({ kind: 'unit', from: 'kg', to: 'lb' });
    expect(parse('5 km')).toMatchObject({ kind: 'units', from: 'km' });
    expect(parse('100 usd to myr')).toEqual({ kind: 'fx', amount: 100, from: 'USD', to: 'MYR' });
    expect(parse('rm 50 in sgd')).toEqual({ kind: 'fx', amount: 50, from: 'MYR', to: 'SGD' });
    expect(parse('$20')).toEqual({ kind: 'fx', amount: 20, from: 'USD', to: null });
    expect(parse('€15 to gbp')).toEqual({ kind: 'fx', amount: 15, from: 'EUR', to: 'GBP' });
  });
  it('says nothing for words or half input', () => {
    expect(parse('5 km to')).toBe(null);
    expect(parse('hello')).toBe(null);
    expect(parse('chrome')).toBe(null);
    expect(parse('5')).toBe(null);
  });
  it('formats answers readably', () => {
    expect(formatNumber(1234567.891)).toBe('1,234,567.891');
    expect(formatNumber(0.1 + 0.2)).toBe('0.3');
    expect(formatNumber(1e21)).toBe('1e+21');
    expect(formatNumber(-0.000012345)).toBe('-0.000012345');
    expect(formatNumber(3.1068559612, 4)).toBe('3.1069');
  });
});
