import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import clsx from 'clsx';
import styles from './index.module.css';

function HeroCta() {
  return (
    <section className={styles.hero}>
      <div className="container">
        <p className={styles.pretitle}>One Rust app, every device, one workflow</p>
        <h1 className={styles.title}>Build native-quality products with Fission</h1>
        <p className={styles.subtitle}>
          Fission gives you a Flutter-style widget model with deterministic state,
          first-class async contracts, and one runtime path across desktop, mobile,
          and WebAssembly.
        </p>
        <div className={styles.ctaRow}>
          <Link className={styles.primaryCta} to="/docs/getting-started/first-app">
            Build your first app
          </Link>
          <Link className={styles.secondaryCta} to="/playground">
            Open interactive playground
          </Link>
        </div>
        <p className={styles.microNote}>
          Fission is under active development. Some platform integrations and tooling
          are still evolving.
        </p>
      </div>
    </section>
  );
}

function Differentiators() {
  const cards = [
    {
      title: 'Deterministic by default',
      body: 'Actions, reducers, and effects are serializable and replayable. State changes stay predictable, testable, and inspectable.',
    },
    {
      title: 'High-performance rendering',
      body: 'GPU-native scene pipeline with stable snapshots and deterministic layout behavior for polished desktop and app experiences.',
    },
    {
      title: 'Commands, services, and jobs',
      body: 'Explicit runtime boundaries for async work and background behavior, with clear lifecycle contracts from app startup to shutdown.',
    },
    {
      title: 'i18n and accessibility from first use',
      body: 'Localization, semantic role/label surfaces, keyboard and focus behavior are treated as integral platform features.',
    },
    {
      title: 'Playground-ready delivery',
      body: 'Worka VM-backed, web-first preview loop lets users edit, run, and validate changes quickly.',
    },
  ];

  return (
    <section className={styles.section}>
      <div className="container">
        <h2 className={styles.sectionTitle}>What matters most at the start</h2>
        <div className={styles.cardGrid}>
          {cards.map((card) => (
            <article className={styles.card} key={card.title}>
              <h3>{card.title}</h3>
              <p>{card.body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

function PlaygroundTeaser() {
  return (
    <section className={styles.section}>
      <div className="container">
        <h2 className={styles.sectionTitle}>Try it in the browser first</h2>
        <p className={styles.sectionCopy}>
          Open a runnable example, change one line, see the result immediately.
          The playground path is intentionally the first step after learning the
          foundations.
        </p>
        <div className={styles.playgroundFrame}>
          <p className={styles.playgroundFallback}>
            Interactive playground embed is booting at runtime from our Worka VM services.
          </p>
        </div>
      </div>
    </section>
  );
}

export default function Home() {
  return (
    <Layout
      title="Fission"
      description="Fission is a Rust UI framework for building production-ready, cross-platform applications from one codebase.">
      <div className={styles.page}>
        <HeroCta />
        <Differentiators />
        <PlaygroundTeaser />
      </div>
    </Layout>
  );
}
