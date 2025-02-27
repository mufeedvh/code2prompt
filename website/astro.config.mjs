// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Code2prompt',
			logo: {
				light:'./src/assets/logo_dark_v0.0.1.svg',
				dark:'./src/assets/logo_light_v0.0.1.svg',
			},
			defaultLocale: 'root',
			locales: {
				// English docs in `src/content/en/`
				root: {
				  label: 'English',
				  lang: 'en',
				},
			},
			social: {
				github: 'https://github.com/mufeedvh/code2prompt',
				discord: 'https://discord.gg/ZZyBbsHTwH',
			},
			sidebar: [
				{
				  label: "🚀 CLI Documentation",
				  items: [
					{ label: "Welcome", link: "docs/cli/welcome" },
					{
					  label: "Tutorials",
					  items: [
						{ label: "Getting Started", link: "docs/cli/tutorials/getting_started" },
						{ label: "Using Templates", link: "docs/cli/tutorials/using_templates" },
						{ label: "Using Glob Pattern Tool", link: "docs/cli/tutorials/using_glob_pattern_tool" },
						{ label: "Use Filter", link: "docs/cli/tutorials/use_filter" },
						{ label: "Handlebars Templates", link: "docs/cli/tutorials/templates" },
					  ],
					},
					{
					  label: "Explanations",
					  items: [
						{ label: "What are Glob Patterns?", link: "docs/cli/explanations/glob_patterns" },
						{ label: "How the Glob Pattern Filter Works", link: "docs/cli/explanations/glob_pattern_filter" },
						{ label: "Understanding Tokenizers", link: "docs/cli/explanations/tokenizers" },
						{ label: "Glob Pattern Tool", link: "docs/cli/explanations/glob_pattern_tool" },
					  ],
					},
					{
					  label: "How-To Guides",
					  items: [
						{ label: "Install Code2Prompt", link: "docs/cli/how_to/install" },
						{ label: "Filter Files", link: "docs/cli/how_to/filter_files" },
						{ label: "Save Generated Prompt", link: "docs/cli/how_to/save_generated_prompt" },
						{ label: "Exclude Files from Tree", link: "docs/cli/how_to/exclude_files_from_tree" },
					  ],
					},
				  ],
				},
				{
				  label: "📦 SDK Documentation",
				  items: [
					{ label: "Welcome", link: "docs/sdk/welcome" },
					{
					  label: "Tutorials",
					  items: [
						{ label: "SDK Tutorial 1", link: "docs/sdk/tutorials/tutorial1" },
						{ label: "SDK Tutorial 2", link: "docs/sdk/tutorials/tutorial2" },
					  ],
					},
					{
					  label: "Explanations",
					  items: [
						{ label: "SDK Concepts", link: "docs/sdk/explanations/sdk_concepts" },
					  ],
					},
					{
					  label: "How-To Guides",
					  items: [
						{ label: "Install SDK", link: "docs/sdk/how_to/install" },
						{ label: "Use SDK", link: "docs/sdk/how_to/use_sdk" },
					  ],
					},
				  ],
				},
			  ]			  
		}),
	],
});
