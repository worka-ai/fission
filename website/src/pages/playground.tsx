import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import {playgroundFlows, repoExamples} from '../data/siteContent';
import styles from './experience.module.css';

const recommendedExamples = repoExamples.filter((example) =>
  ['counter', 'widget-gallery', 'text-lab', 'web-smoke', 'mobile-smoke'].includes(example.slug),
);

function repoHref(path: string) {
  return `https://github.com/worka-ai/fission/tree/main/${path}`;
}

export default function Playground() {
  return (
    <Layout title='Playground' description='The current practical path for experimenting with Fission.'>
      <main className={`container ${styles.pageShell}`}>
        <section className={styles.section}>
          <h1 className={styles.heading}>Playground</h1>
          <p className={styles.lead}>
            Today, Fission does not ship a live in-browser editor for trying widgets from a text box in your browser. The practical playground is local and target-aware: run a small example or your own app on the host that best matches the product question you are exploring, change one reducer or one widget branch, and immediately see the result in the real runtime you will keep using later.
          </p>
          <div className={styles.heroPanel}>
            <div className={styles.heroCopy}>
              <p>
                That may sound less glamorous than a toy sandbox, but it is more honest and more useful. You are experimenting against the real layout engine, the real input pipeline, the real rendering path, and the real platform hosts. When something works in this loop, you are learning something that carries directly into product code.
              </p>
              <p>
                Choose the smallest host that can answer your question. On many machines, desktop is the easiest general loop for shared-runtime questions because rebuilds are fast and packaging friction is low. But if your product question is really about browser hosting, Android packaging, iOS simulator flow, viewport behavior, or another target-specific concern, start on that host instead. The point of the playground is not to define a primary platform. It is to give you a truthful experiment loop.
              </p>
            </div>
            <aside className={styles.heroAside}>
              <p className={styles.smallHeading}>What this page is for</p>
              <p>
                Use it to choose a safe experiment loop today, based on the kind of app behavior or host behavior you need to prove.
              </p>
              <Link className={styles.link} to='/docs/learn/quickstart'>
                Start with Quickstart
              </Link>
            </aside>
          </div>
        </section>

        <section className={styles.section}>
          <p className={styles.eyebrow}>Recommended loops</p>
          <h2 className={styles.heading}>Pick the smallest loop that answers your question</h2>
          <p className={styles.lead}>
            Different loops are useful for different kinds of work. If you only need to understand a reducer or a layout branch, keep the loop small. If you need to verify overlays, semantics, screenshots, packaging, or a target launcher, widen the loop deliberately and choose the host that can actually prove that behavior.
          </p>
          <div className={styles.storyGrid}>
            {playgroundFlows.map((flow) => (
              <article className={styles.storyCard} key={flow.title}>
                <h2>{flow.title}</h2>
                <div className={styles.storyBody}>
                  <p>{flow.summary}</p>
                  {flow.followUp ? <p>{flow.followUp}</p> : null}
                </div>
                <div className={styles.commandStack}>
                  {flow.commands.map((command) => (
                    <code className={styles.commandBlock} key={command}>
                      {command}
                    </code>
                  ))}
                </div>
              </article>
            ))}
          </div>
        </section>

        <section className={styles.section}>
          <p className={styles.eyebrow}>Good scratchpads</p>
          <h2 className={styles.heading}>These examples make especially good places to experiment</h2>
          <p className={styles.lead}>
            Each example below is small enough to edit safely and focused enough that one change usually teaches one idea. That makes them better playgrounds than jumping straight into the largest product-like crate. If your question lives on a browser or mobile host, the target examples belong in that first group too.
          </p>
          <div className={styles.storyGrid}>
            {recommendedExamples.map((example) => (
              <article className={styles.storyCard} key={example.slug}>
                <div className={styles.metaRow}>
                  <span className={styles.pill}>{example.crate}</span>
                  <code>{example.repoPath}</code>
                </div>
                <h2>{example.title}</h2>
                <div className={styles.storyBody}>
                  <p>{example.summary}</p>
                  <p>
                    Use this when you want to practice{' '}
                    {example.features.length === 1
                      ? example.features[0]
                      : `${example.features.slice(0, -1).join(', ')}, and ${example.features.at(-1)}`}
                    .
                  </p>
                </div>
                <div className={styles.commandStack}>
                  {example.commands.map((command) => (
                    <code className={styles.commandBlock} key={command}>
                      {command}
                    </code>
                  ))}
                </div>
                <div className={styles.linkRow}>
                  <Link className={styles.link} to={repoHref(example.repoPath)}>
                    Open repo path
                  </Link>
                  {example.docsHref ? (
                    <Link className={styles.link} to={example.docsHref}>
                      Related docs
                    </Link>
                  ) : null}
                </div>
              </article>
            ))}
          </div>
        </section>

        <section className={styles.section}>
          <div className={styles.sectionPanel}>
            <p className={styles.eyebrow}>Later</p>
            <h2>What a future browser playground would add</h2>
            <div className={styles.sectionBody}>
              <p>
                A future browser playground would make sharing snippets and trying small layout ideas easier. That would be useful, especially for teaching and quick experiments.
              </p>
              <p>
                Until that exists, the honest path is still to experiment in the real hosts you plan to learn or ship on. That keeps you close to the checked-in examples, teaches the actual runtime and host boundary, and avoids a split between a toy sandbox and the architecture you actually ship.
              </p>
              <div className={styles.linkRow}>
                <Link className={styles.link} to='/examples'>
                  Browse examples
                </Link>
                <Link className={styles.link} to='/docs/learn/examples-and-targets'>
                  Read about examples and targets
                </Link>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
