import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// Pushrod documentation site. Run `npm start` for a live preview and
// `npm run build` to check that every page and link builds.

const config: Config = {
  title: 'Pushrod',
  tagline: 'A small, simple, GPU-drawn GUI library for Rust, built on SDL2',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://kensuenobu.github.io',
  baseUrl: '/rust-pushrod/',

  organizationName: 'KenSuenobu',
  projectName: 'rust-pushrod',

  onBrokenLinks: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          routeBasePath: '/',
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/KenSuenobu/rust-pushrod/tree/main/pushrod-docs/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Pushrod',
      items: [
        {type: 'docSidebar', sidebarId: 'docs', position: 'left', label: 'Documentation'},
        {href: 'https://kensuenobu.github.io/rust-pushrod/api/pushrod/', label: 'API reference', position: 'left', target: '_self'},
        {to: '/getting-started/quick-start', label: 'Quick start', position: 'left'},
        {
          href: 'https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md',
          label: 'Roadmap',
          position: 'right',
        },
        {href: 'https://github.com/KenSuenobu/rust-pushrod', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      style: 'light',
      links: [
        {
          title: 'Documentation',
          items: [
            {label: 'Quick start', to: '/getting-started/quick-start'},
            {label: 'Creating an application', to: '/guides/creating-an-application'},
            {label: 'Widgets', to: '/widgets'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'GitHub', href: 'https://github.com/KenSuenobu/rust-pushrod'},
            {label: 'Issues', href: 'https://github.com/KenSuenobu/rust-pushrod/issues'},
            {
              label: 'Roadmap',
              href: 'https://github.com/KenSuenobu/rust-pushrod/blob/main/docs/ROADMAP_EXTENSIBLE_WIDGET_LIBRARY.md',
            },
          ],
        },
      ],
      copyright: `Pushrod is licensed under the Apache License, Version 2.0.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
