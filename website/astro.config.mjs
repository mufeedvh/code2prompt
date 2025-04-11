// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import react from "@astrojs/react";
import remarkMath from "remark-math";
import rehypeMathjax from "rehype-mathjax";
import mdx from "@astrojs/mdx";
import tailwindcss from "@tailwindcss/vite";
import sitemap from "@astrojs/sitemap";
import starlightBlog from "starlight-blog";

import { passthroughImageService } from "astro/config";

// https://astro.build/config
export default defineConfig({
  site: "https://code2prompt.dev",
  integrations: [
    starlight({
      title: "Code2prompt",
      logo: {
        light: "./src/assets/logo_dark_v0.0.1.svg",
        dark: "./src/assets/logo_light_v0.0.1.svg",
      },
      defaultLocale: "root",
      locales: {
        // English docs in `src/content/en/`
        root: {
          label: "English",
          lang: "en",
        },
      },
      social: [
        {
          icon: "discord",
          label: "Discord",
          href: "https://discord.gg/ZZyBbsHTwH",
        },
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/mufeedvh/code2prompt",
        },
      ],
      sidebar: [
        {
          label: "Documentation 🚀 ",
          items: [
            {
              label: "Tutorials",
              items: [
                {
                  label: "Getting Started",
                  link: "docs/tutorials/getting_started",
                },
                {
                  label: "Learn Templating",
                  link: "docs/tutorials/learn_templates",
                },
                {
                  label: "Learn Filtering",
                  link: "docs/tutorials/learn_filters",
                },
              ],
            },
            {
              label: "Explanations",
              items: [
                {
                  label: "What are Glob Patterns?",
                  link: "docs/explanations/glob_patterns",
                },
                {
                  label: "How the Glob Pattern Filter Works",
                  link: "docs/explanations/glob_pattern_filter",
                },
                {
                  label: "Understanding Tokenizers",
                  link: "docs/explanations/tokenizers",
                },
              ],
            },
            {
              label: "How-To Guides",
              items: [
                { label: "Install Code2Prompt", link: "docs/how_to/install" },
                { label: "Filter Files", link: "docs/how_to/filter_files" },
              ],
            },
          ],
        },
        { label: "Welcome 👋", link: "docs/welcome" },
        {
          label: "Vision 🔮",
          link: "docs/vision",
        },
      ],
      plugins: [starlightBlog()],
    }),
    react(),
    mdx(),
    sitemap(),
  ],

  markdown: {
    remarkPlugins: [remarkMath],
    rehypePlugins: [rehypeMathjax],
  },

  vite: {
    plugins: [tailwindcss()],
  },
  image: {
    service: passthroughImageService(),
  },
});
