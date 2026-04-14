// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import astroMermaid from 'astro-mermaid';

export default defineConfig({
	site: 'https://mde-pach.github.io',
	base: '/nrs',
	integrations: [
		starlight({
			title: 'NRS',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/mde-pach/nrs' }],
			sidebar: [
				{ label: 'Overview', slug: 'overview' },
				{ label: 'Quick Start', slug: 'quickstart' },
				{
					label: 'Concepts',
					items: [
						{ label: 'Context Layers', slug: 'concepts/layers' },
						{ label: 'Rules', slug: 'concepts/rules' },
						{ label: 'Workflow', slug: 'concepts/workflow' },
						{ label: 'Lifecycle', slug: 'concepts/lifecycle' },
					],
				},
				{
					label: 'CLI',
					items: [
						{ label: 'Commands', slug: 'cli/commands' },
						{ label: 'Skill', slug: 'cli/skill' },
					],
				},
				{ label: 'Example', slug: 'example' },
				{ label: 'Research', slug: 'research' },
			],
		}),
		astroMermaid(),
	],
});
