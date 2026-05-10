// @vitest-environment jsdom

import { render, screen } from '@testing-library/svelte';
import EmptyState from './EmptyState.svelte';

describe('EmptyState', () => {
	it('renders the title', () => {
		render(EmptyState, { props: { title: 'Nothing here' } });
		expect(screen.getByRole('heading', { name: 'Nothing here' })).toBeInTheDocument();
	});

	it('renders the description when provided', () => {
		render(EmptyState, { props: { title: 'Empty', description: 'Try adding a feed' } });
		expect(screen.getByText('Try adding a feed')).toBeInTheDocument();
	});

	it('does not render description when omitted', () => {
		render(EmptyState, { props: { title: 'Empty' } });
		expect(screen.queryByRole('paragraph')).not.toBeInTheDocument();
	});

	it('renders overline when provided', () => {
		render(EmptyState, { props: { title: 'Empty', overline: 'STATUS' } });
		expect(screen.getByText('STATUS')).toBeInTheDocument();
	});

	it('renders without error when icon prop is provided', () => {
		const { container } = render(EmptyState, { props: { title: 'Empty', icon: 'mdi:rss' } });
		expect(container).toBeTruthy();
		expect(screen.getByRole('heading', { name: 'Empty' })).toBeInTheDocument();
	});

	it('applies custom class names', () => {
		render(EmptyState, {
			props: {
				title: 'Empty',
				class: 'custom-container',
				titleClass: 'custom-title'
			}
		});
		const heading = screen.getByRole('heading', { name: 'Empty' });
		expect(heading).toHaveClass('custom-title');
	});
});
