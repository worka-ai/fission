import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import styles from './experience.module.css';

const examples = [
  {
    title: 'Counter',
    summary: 'A compact, deterministic state example with clear reducer flow.',
    doc: '/docs/tutorials/counter',
  },
  {
    title: 'Todo',
    summary: 'State, commands, and services with real app-like behavior.',
    doc: '/docs/tutorials/todo',
  },
  {
    title: 'Accessible dashboard',
    summary: 'High-signal product layout with localization and a11y-oriented patterns.',
    doc: '/docs/guide/i18n-and-accessibility',
  },
];

export default function Examples() {
  return (
    <Layout title="Examples" description="Explore Fission examples.">
      <main className={`container ${styles.pageShell}`}>
        <section className={styles.section}>
          <h1 className={styles.heading}>Examples</h1>
          <p className={styles.lead}>
            Product-forward examples linked to both narrative tutorials and API references.
          </p>
        </section>
        <section className={styles.section}>
          <div className={styles.grid}>
            {examples.map((item) => (
              <article className={styles.card} key={item.title}>
                <h2>{item.title}</h2>
                <p>{item.summary}</p>
                <Link className={styles.link} to={item.doc}>
                  Start this example
                </Link>
              </article>
            ))}
          </div>
        </section>
      </main>
    </Layout>
  );
}
