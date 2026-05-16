import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import styles from './experience.module.css';

function DemoCard({title, description, url}: {title: string; description: string; url: string}) {
  return (
    <article className={styles.card}>
      <h2>{title}</h2>
      <p>{description}</p>
      <Link className={styles.link} to={url}>
        Open sample
      </Link>
    </article>
  );
}

export default function Playground() {
  return (
    <Layout title="Playground" description="Live examples and iterative editing with Fission.">
      <main className={`container ${styles.pageShell}`}>
        <section className={styles.section}>
          <h1 className={styles.heading}>Playground</h1>
          <p className={styles.lead}>
            Use the web preview to validate behavior quickly, then move stable work into
            local projects.
          </p>
        </section>
        <section className={styles.section}>
          <h2>Starter examples</h2>
          <div className={styles.grid}>
            <DemoCard
              title="Counter"
              description="Signals, reducers, and rebuild loop in one compact flow."
              url="/docs/tutorials/counter"
            />
            <DemoCard
              title="Todo"
              description="Commands, services, and async persistence behavior."
              url="/docs/tutorials/todo"
            />
            <DemoCard
              title="Responsive layout"
              description="Wide-screen dashboard behavior and responsive composition."
              url="/docs/guide/widgets-and-layout"
            />
          </div>
        </section>
      </main>
    </Layout>
  );
}
