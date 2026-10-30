// A small calculator for the search panel: a tokenizer and a recursive-descent parser over
// numbers, + − × ÷ ^ mod, brackets, percentages and a few functions. Never eval: an unknown
// word or a half-typed expression is just "no answer" (null). Tested in calc.test.ts.
//
//   expr    = term (('+' | '-') term)*         a + b%  means a plus b percent of a
//   term    = power (('*' | '/' | 'mod' | 'of') power | power-without-operator)*
//   power   = unary ('^' power)?               right-associative
//   unary   = '-' unary | postfix
//   postfix = primary '%'?
//   primary = number | constant | function '(' args ')' | '(' expr ')'

type Tok = { t: 'num'; v: number } | { t: 'op'; v: string } | { t: 'id'; v: string };

const DEG = Math.PI / 180;
const FUNCS = new Map<string, (...a: number[]) => number>([
  ['sqrt', Math.sqrt], ['cbrt', Math.cbrt], ['abs', Math.abs], ['floor', Math.floor], ['ceil', Math.ceil], ['exp', Math.exp],
  ['ln', Math.log], ['log', (x, b) => (b === undefined ? Math.log10(x) : Math.log(x) / Math.log(b))],
  ['sin', (x) => Math.sin(x * DEG)], ['cos', (x) => Math.cos(x * DEG)], ['tan', (x) => Math.tan(x * DEG)],
  ['round', (x, n = 0) => { const k = 10 ** n; return Math.round(x * k) / k; }],
  ['min', Math.min], ['max', Math.max],
]);
const CONSTS = new Map<string, number>([['pi', Math.PI], ['π', Math.PI], ['e', Math.E]]);

function tokenize(src: string): Tok[] | null {
  const s = src.toLowerCase()
    .replace(/(\d),(?=\d{3}(?!\d))/g, '$1') // 1,000 -> 1000 (but "round(2.5, 2)" keeps its comma)
    .replace(/[×✕∗]/g, '*').replace(/÷/g, '/').replace(/−/g, '-');
  const out: Tok[] = [];
  let i = 0;
  while (i < s.length) {
    const c = s[i];
    if (c === ' ') { i++; continue; }
    const num = /^(\d+\.?\d*|\.\d+)(e[+-]?\d+)?/.exec(s.slice(i));
    if (num) { out.push({ t: 'num', v: parseFloat(num[0]) }); i += num[0].length; continue; }
    const id = /^[a-zπ]+/.exec(s.slice(i));
    if (id) {
      // "x" between things is "times"; "mod" and "of" are operators.
      out.push(id[0] === 'x' ? { t: 'op', v: '*' } : id[0] === 'mod' || id[0] === 'of' ? { t: 'op', v: id[0] } : { t: 'id', v: id[0] });
      i += id[0].length; continue;
    }
    if ('+-*/^%(),'.includes(c)) { out.push({ t: 'op', v: c === '*' && s[i + 1] === '*' ? (i++, '^') : c }); i++; continue; }
    return null;
  }
  return out;
}

/** The answer, or null when the input isn't (yet) a calculation. A bare number isn't one. */
export function evaluate(src: string): number | null {
  const tk = tokenize(src);
  if (!tk || !tk.length) return null;
  const toks: Tok[] = tk;
  let i = 0;
  let worked = false; // an operator, function, constant or % was used
  const peek = () => toks[i];
  const isOp = (v: string) => peek()?.t === 'op' && peek()!.v === v;
  const fail = (): never => { throw 0; };

  // A value and whether it was written as a percentage (for "a + b%").
  type V = { v: number; pct: boolean };
  function expr(): number {
    let a = term().v;
    while (isOp('+') || isOp('-')) {
      const op = (toks[i++] as { v: string }).v;
      const b = term();
      worked = true;
      const d = b.pct ? a * b.v : b.v;
      a = op === '+' ? a + d : a - d;
    }
    return a;
  }
  function term(): V {
    let a = power();
    for (;;) {
      if (isOp('*') || isOp('/') || isOp('mod') || isOp('of')) {
        const op = (toks[i++] as { v: string }).v;
        const b = power().v;
        worked = true;
        a = { v: op === '/' ? a.v / b : op === 'mod' ? a.v % b : a.v * b, pct: false };
      } else if (peek() && (peek()!.t === 'id' || isOp('(')) ) {
        a = { v: a.v * power().v, pct: false }; // 2(3+1), 2pi
        worked = true;
      } else return a;
    }
  }
  function power(): V {
    const a = unary();
    if (!isOp('^')) return a;
    i++; worked = true;
    return { v: a.v ** power().v, pct: false };
  }
  function unary(): V {
    if (isOp('-')) { i++; const a = unary(); return { v: -a.v, pct: a.pct }; }
    if (isOp('+')) { i++; return unary(); }
    const a = primary();
    if (isOp('%')) { i++; worked = true; return { v: a / 100, pct: true }; }
    return { v: a, pct: false };
  }
  function primary(): number {
    const t = toks[i++] ?? fail();
    if (t.t === 'num') return t.v;
    if (t.t === 'op' && t.v === '(') { const v = expr(); if (!isOp(')')) fail(); i++; return v; }
    if (t.t === 'id') {
      const k = CONSTS.get(t.v);
      if (k !== undefined) { worked = true; return k; }
      const f = FUNCS.get(t.v) ?? fail();
      if (!isOp('(')) fail();
      i++;
      const args = [expr()];
      while (isOp(',')) { i++; args.push(expr()); }
      if (!isOp(')')) fail();
      i++; worked = true;
      return f(...args);
    }
    return fail();
  }
  try {
    const v = expr();
    if (i !== toks.length || !worked || !Number.isFinite(v)) return null;
    return v;
  } catch {
    return null;
  }
}
