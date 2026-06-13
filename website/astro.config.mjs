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
  redirects: {
    "/fr": "/",
    "/de": "/",
    "/es": "/",
    "/zh": "/",
    "/ja": "/",
    "/ru": "/",
  },
  integrations: [
    starlight({
      title: "Code2prompt",
      logo: {
        light: "./src/assets/logo_dark_v0.0.1.svg",
        dark: "./src/assets/logo_light_v0.0.1.svg",
      },
      defaultLocale: "root",
      locales: {
        // English docs in `src/content/docs/`
        root: {
          label: "English",
          lang: "en",
        },
        // French docs in `src/content/docs/fr/docs/`
        fr: {
          label: "Français",
          lang: "fr",
        },
        // German docs in `src/content/docs/de/docs/`
        de: {
          label: "Deutsch",
          lang: "de",
        },
        // Spanish docs in `src/content/docs/es/docs/`
        es: {
          label: "Español",
          lang: "es",
        },
        // Chinese docs in `src/content/docs/zh/docs/`
        zh: {
          label: "中文",
          lang: "zh",
        },
        // Japanese docs in `src/content/docs/ja/docs/`
        ja: {
          label: "日本語",
          lang: "ja",
        },
        // Russian docs in `src/content/docs/ru/docs/`
        ru: {
          label: "Русский",
          lang: "ru",
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
                { 
                  label: "Learn Configuration", 
                  link: "docs/tutorials/configuration"
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
                { label: "Install Code2prompt", link: "docs/how_to/install" },
                { label: "Basic Usage", link: "docs/how_to/cli" },
                { label: "Configuration File", link: "docs/how_to/config" },
                { label: "Use with SSH", link: "docs/how_to/ssh" },
              ],
            },
            {
              label: "Reference",
              items: [
                { label: 'API Rust Docs', link: 'https://docs.rs/code2prompt_core' },
              ]},
          ],
        },
        { label: "Welcome 👋", link: "docs/welcome" },
        {
          label: "Vision 🔮",
          link: "docs/vision",
        },
      ],
      plugins: [
        starlightBlog({
          authors: {
            ODAncona: {
              name: "Olivier D'Ancona",
              title: "Data Scientist",
              picture: "assets/images/odancona.png",
              url: "https://www.linkedin.com/in/odancona/",
            },
          },
        }),
      ],
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
