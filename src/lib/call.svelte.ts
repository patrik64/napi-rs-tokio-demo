/** Reactive state for one request at a time: is it running, and how did the last one end? */
export class Call<T> {
	pending = $state(false);
	result = $state.raw<T>();
	error = $state<string>();

	async run(request: () => Promise<T>): Promise<void> {
		this.pending = true;
		this.error = undefined;
		try {
			this.result = await request();
		} catch (error) {
			this.result = undefined;
			this.error = error instanceof Error ? error.message : String(error);
		} finally {
			this.pending = false;
		}
	}
}
