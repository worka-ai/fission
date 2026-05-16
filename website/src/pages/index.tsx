import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import styles from './index.module.css';
import type { ReactNode } from 'react';

type Principle = {
  title: string;
  body: ReactNode;
};

const principles: Principle[] = [
  {
    title: 'Desktop-first baseline',
    body: (
      <>
        Run <code>cargo run</code> from the generated app first, then expand behavior once desktop
        interactions are stable.
      </>
    ),
  },
  {
    title: 'Generated host expansion',
    body: (
      <>
        Add <code>web</code>, <code>ios</code>, and <code>android</code> targets after app logic is stable,
        then launch through <code>platforms/&lt;target&gt;/run-*.sh</code>.
      </>
    ),
  },
  {
    title: 'Production-first workflows',
    body: 'Keep async work in jobs, services, and commands while reducers stay pure and deterministic.',
  },
  {
    title: 'Wide-screen layout first',
    body: 'Start from explicit containers (SafeArea/Container/Column/Row), then add actions and polish in a stable visual skeleton.',
  },
];

function Hero() {
  return (
    <section className={styles.hero}>
      <div className='container'>
        <p className={styles.kicker}>Rust-powered UI for product teams</p>
        <h1 className={styles.title}>Build one app model, validate quickly, then ship to every runtime.</h1>
        <p className={styles.subtitle}>
          Fission gives you a single state-driven model in Rust and generated host projects for desktop,
          web, iOS, and Android. Learn the flow once in one place, then move it to more platforms
          without re-architecting your state or interaction layer.
        </p>
        <div className={styles.ctaRow}>
          <Link className={styles.primaryCta} to='/docs/getting-started/first-app'>
            Build your first app
          </Link>
          <Link className={styles.secondaryCta} to='/docs/getting-started/install'>
            Install the CLI
          </Link>
          <Link className={styles.tertiaryCta} to='/docs/guide/commands-services-jobs'>
            Add async behavior
          </Link>
        </div>
      </div>
    </section>
  );
}

function ProblemSolution() {
  return (
    <section className={styles.section}>
      <div className='container'>
        <div className={styles.storyPanel}>
          <p className={styles.storyLabel}>When products scale</p>
          <p className={styles.storyTitle}>You get speed without losing control.</p>
          <p className={styles.storyCopy}>
            Fission keeps UI state, behavior, and rendering in one deterministic loop.
            For teams shipping production apps, this creates a stable loop:
            edit state logic, rebuild, verify behavior, then add the next target.
          </p>
          <p className={styles.storyCopy}>
            Platform output is generated as needed. Read each target readme and run its script when you
            are ready to expand from desktop confidence to web or mobile confidence.
          </p>
        </div>

        <div className={styles.cardGrid}>
          {principles.map((principle) => (
            <article className={styles.card} key={principle.title}>
              <h2>{principle.title}</h2>
              <p>{principle.body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

function ProofStrip() {
  return (
    <section className={styles.section}>
      <div className='container'>
        <p className={styles.sectionLead}>Use this validation order for fewer platform surprises:</p>
        <div className={styles.proofGrid}>
          <p>
            <strong>Start</strong> with a desktop-first workflow: <Link to='/docs/guide/widgets-and-layout'>widgets + layout</Link>,
            <Link to='/docs/guide/state-and-actions'> state + actions</Link>, then
            <Link to='/docs/guide/commands-services-jobs'> async boundaries</Link>.
          </p>
          <p>
            <strong>Validate</strong> each interaction in the running app first with <code>cargo run</code>.
            Confirm reducer outputs and focus order before adding commands, services, or jobs.
          </p>
          <p>
            <strong>Expand</strong> with generated targets only after parity: run
            <code>./platforms/web/run-browser.sh</code>,
            <code>./platforms/ios/run-sim.sh</code>,
            <code>./platforms/android/run-emulator.sh</code> as applicable, while honoring toolchain prerequisites.
          </p>
        </div>
      </div>
    </section>
  );
}

export default function Home() {
  return (
    <Layout
      title='Fission'
      description='Rust UI framework for desktop, web, iOS, and Android with deterministic state and production-focused workflows.'>
      <div className={styles.page}>
        <Hero />
        <ProblemSolution />
        <ProofStrip />
      </div>
    </Layout>
  );
}
