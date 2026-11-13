/** Parses a JSON request body, treating anything that isn't an object as empty. */
export async function readJson(request: Request): Promise<Record<string, unknown>> {
	const body: unknown = await request.json().catch(() => null);
	return body !== null && typeof body === 'object' ? (body as Record<string, unknown>) : {};
}

/** Reads `value` as a whole number within `[min, max]`, or returns `fallback`. */
export function int(value: unknown, min: number, max: number, fallback: number): number {
	if (value === null || value === undefined || value === '') return fallback;
	const number = Number(value);
	return Number.isFinite(number) ? Math.min(max, Math.max(min, Math.round(number))) : fallback;
}
