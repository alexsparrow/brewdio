import { getPermalink } from './utils/permalinks';

export const headerData = {
  links: [
    {
      text: 'Features',
      href: getPermalink('/#features'),
    },
    {
      text: 'Pricing',
      href: getPermalink('/pricing'),
    },
  ],
  actions: [
    { text: 'GitHub', href: 'https://github.com/brewdio/brewdio', target: '_blank' as const, icon: 'tabler:brand-github', variant: 'secondary' as const },
    { text: 'Open Web App', href: 'https://app.brewdio.beer', variant: 'primary' as const },
  ],
};

export const footerData = {
  links: [
    {
      title: 'Product',
      links: [
        { text: 'Features', href: getPermalink('/#features') },
        { text: 'Pricing', href: getPermalink('/pricing') },
      ],
    },
    {
      title: 'Resources',
      links: [
        { text: 'GitHub', href: 'https://github.com/brewdio/brewdio' },
        { text: 'Documentation', href: '#' },
      ],
    },
  ],
  secondaryLinks: [
    { text: 'License: MIT', href: 'https://github.com/brewdio/brewdio/blob/main/LICENSE' },
  ],
  socialLinks: [
    { ariaLabel: 'Github', icon: 'tabler:brand-github', href: 'https://github.com/brewdio/brewdio' },
  ],
  footNote: `
    <a class="text-blue-600 underline dark:text-muted" href="https://app.brewdio.beer">brewdio</a> — Open-source brewing software. MIT License.
  `,
};
