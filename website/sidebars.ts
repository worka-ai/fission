import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  learnSidebar: [
    {
      type: 'doc',
      id: 'getting-started/what-is-fission',
    },
    {
      type: 'category',
      label: 'Get Started',
      collapsed: false,
      items: [
        'getting-started/our-differentiators',
        'getting-started/install',
        'getting-started/first-app',
      ],
    },
    {
      type: 'category',
      label: 'Build your first apps',
      collapsed: false,
      items: ['tutorials/counter', 'tutorials/todo'],
    },
    {
      type: 'category',
      label: 'Guide',
      collapsed: false,
      items: [
        'guide/widgets-and-layout',
        'guide/state-and-actions',
        'guide/commands-services-jobs',
        'guide/i18n-and-accessibility',
        'guide/platform-deployment',
        'guide/playground-driven-workflow',
      ],
    },
    {
      type: 'category',
      label: 'Concepts',
      collapsed: false,
      items: ['concepts/widget-tree', 'concepts/pipeline', 'concepts/testing-and-observability'],
    },
  ],
};

export default sidebars;
