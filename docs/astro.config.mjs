import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://docs.brewdio.beer',
  integrations: [
    starlight({
      title: 'brewdio',
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Installation', slug: 'getting-started/installation' },
            { label: 'Quick Start', slug: 'getting-started/quick-start' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Recipes', slug: 'guides/recipes' },
            { label: 'Batches', slug: 'guides/batches' },
          ],
        },
        {
          label: 'Data Model & Calculations',
          items: [
            { label: 'Overview', slug: 'calculations/overview' },
            { label: 'Original Gravity', slug: 'calculations/original-gravity' },
            { label: 'Final Gravity', slug: 'calculations/final-gravity' },
            { label: 'ABV', slug: 'calculations/abv' },
            { label: 'IBU', slug: 'calculations/ibu' },
            { label: 'Color (SRM)', slug: 'calculations/color' },
            { label: 'Carbonation', slug: 'calculations/carbonation' },
            { label: 'Water', slug: 'calculations/water' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'BeerJSON', slug: 'reference/beerjson' },
            { label: 'API', slug: 'reference/api' },
          ],
        },
      ],
    }),
  ],
});
