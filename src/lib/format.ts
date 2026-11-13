const whole = new Intl.NumberFormat('en-US', { maximumFractionDigits: 0 });
const tenths = new Intl.NumberFormat('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 });

/** `1234.5` becomes `1,235`. */
export const count = (value: number) => whole.format(value);

/** Milliseconds, with a decimal only where it matters: `0.4 ms`, `1,235 ms`. */
export const ms = (value: number) => `${(value < 10 ? tenths : whole).format(value)} ms`;
