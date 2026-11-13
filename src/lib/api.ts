/** POSTs JSON to one of the demo endpoints and returns the parsed reply. */
export async function post<T>(path: string, body: unknown): Promise<T> {
	const response = await fetch(path, {
		method: 'POST',
		headers: { 'content-type': 'application/json', accept: 'application/json' },
		body: JSON.stringify(body)
	});

	if (!response.ok) {
		const problem: { message?: string } | null = await response.json().catch(() => null);
		throw new Error(problem?.message ?? `Request failed with status ${response.status}`);
	}
	return response.json();
}
