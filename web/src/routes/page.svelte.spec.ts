import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import Page from './+page.svelte';

describe('/+page.svelte', () => {
	it('should render game canvas', () => {
		render(Page);

		const canvas = document.querySelector('#canvas');
		expect(canvas).not.toBeNull();
	});
});
