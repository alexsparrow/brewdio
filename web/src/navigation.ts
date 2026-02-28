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
    { text: 'GitHub', href: '#', target: '_blank' as const, icon: 'tabler:brand-github', variant: 'secondary' as const },
    { text: 'Open Web App', href: '#', variant: 'primary' as const },
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
        { text: 'GitHub', href: '#' },
        { text: 'Documentation', href: '#' },
      ],
    },
  ],
  secondaryLinks: [
    { text: 'License: MIT', href: '#' },
  ],
  socialLinks: [
    { ariaLabel: 'Github', icon: 'tabler:brand-github', href: '#' },
  ],
  footNote: `
    <a class="text-blue-600 underline dark:text-muted" href="#">brewdio</a> — Open-source brewing software. MIT License.
  `,
};
