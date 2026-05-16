import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  referenceSidebar: [
    {
      type: 'doc',
      id: 'overview/overview',
    },
    {
      type: 'category',
      label: 'Core Framework',
      collapsed: false,
      items: [
        'core/widget-trait',
        'core/state-system',
        'core/commands-services-jobs',
        'core/rendering-pipeline',
      ],
    },
    {
      type: 'category',
      label: 'Widgets',
      collapsed: false,
      items: ['widgets/catalog', 'widgets/layout', 'widgets/inputs', 'widgets/media'],
    },
    {
      type: 'category',
      label: 'Platform and Tooling',
      collapsed: false,
      items: ['platform/targets', 'platform/accessibility-and-i18n', 'platform/testing', 'cli/overview'],
    },
    {
      type: 'category',
      label: 'Worka VM',
      collapsed: false,
      items: ['worka/overview', 'worka/playground', 'worka/bundles'],
    },
  ],
};

export default sidebars;
