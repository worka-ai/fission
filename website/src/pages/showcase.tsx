import Layout from '@theme/Layout';
import styles from './experience.module.css';

const stories = [
  {
    title: 'Core playground sample',
    description: 'Focused demo for deterministic updates and render stability.',
  },
  {
    title: 'Widget gallery',
    description: 'Design system-first composition patterns for production UI.',
  },
  {
    title: 'Command workflow app',
    description: 'Command/service/job orchestration shown in a real product flow.',
  },
];

export default function Showcase() {
  return (
    <Layout title="Showcase" description="Fission apps and practical implementation stories.">
      <main className={`container ${styles.pageShell}`}>
        <section className={styles.section}>
          <h1 className={styles.heading}>Showcase</h1>
          <p className={styles.lead}>
            We are building out a broader showcase catalog. Current stories prioritize
            production-oriented architecture and interaction quality.
          </p>
        </section>
        <section className={styles.section}>
          {stories.map((story) => (
            <article className={styles.card} key={story.title}>
              <h2>{story.title}</h2>
              <p>{story.description}</p>
            </article>
          ))}
        </section>
      </main>
    </Layout>
  );
}
