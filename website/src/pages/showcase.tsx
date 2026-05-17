import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import {showcaseStories} from '../data/siteContent';
import styles from './experience.module.css';

function repoHref(path: string) {
  return `https://github.com/worka-ai/fission/tree/main/${path}`;
}

export default function Showcase() {
  return (
    <Layout title='Showcase' description='Concrete, repo-backed examples that prove what Fission already supports.'>
      <main className={`container ${styles.pageShell}`}>
        <section className={styles.section}>
          <h1 className={styles.heading}>Showcase</h1>
          <p className={styles.lead}>
            This page is intentionally narrow. It is not a gallery of aspirational mockups. It is a short list of checked-in examples that prove meaningful parts of the framework already work in code you can run today.
          </p>
          <div className={styles.heroPanel}>
            <div className={styles.heroCopy}>
              <p>
                Each story here answers a skeptical question. Can Fission handle a real app shell instead of a counter? Can it host custom rendering? Can it survive text-input edge cases? Can it actually launch through browser and mobile hosts instead of only claiming target support in theory?
              </p>
              <p>
                The point of showcase examples is not only that they look substantial. The point is that each one exercises architecture that matters in production: explicit state, deterministic flow, runtime-managed resources, portals, rendering boundaries, and platform hosts.
              </p>
            </div>
            <aside className={styles.heroAside}>
              <p className={styles.smallHeading}>Good next stop</p>
              <p>
                After reading a story here, open the matching guide to understand why the example is built that way instead of treating the repo as the explanation.
              </p>
              <Link className={styles.link} to='/docs/learn/overview'>
                Read the overview
              </Link>
            </aside>
          </div>
        </section>

        <section className={styles.section}>
          <p className={styles.eyebrow}>Repo-backed stories</p>
          <h2 className={styles.heading}>What these examples prove</h2>
          <p className={styles.lead}>
            The strongest showcase examples are not just larger. They isolate different architectural claims and make those claims inspectable through source, tests, and host launch paths.
          </p>
          <div className={styles.storyGrid}>
            {showcaseStories.map((story, index) => (
              <article
                className={`${styles.storyCard} ${index === 1 ? styles.storyCardWide : ''}`}
                key={story.title}>
                <div className={styles.metaRow}>
                  <span className={styles.pill}>repo-backed</span>
                  <code>{story.repoPath}</code>
                </div>
                <h2>{story.title}</h2>
                <div className={styles.storyBody}>
                  <p>{story.summary}</p>
                  <p>
                    This matters because it shows the shared runtime solving a real class of product problem instead of only rendering isolated widgets. The example is useful when you need a concrete proof, but the related guide explains the design choices in a slower and more beginner-friendly way.
                  </p>
                </div>
                <ul className={styles.detailList}>
                  {story.proofs.map((proof) => (
                    <li key={proof}>{proof}</li>
                  ))}
                </ul>
                <div className={styles.linkRow}>
                  <Link className={styles.link} to={repoHref(story.repoPath)}>
                    Open repo path
                  </Link>
                  <Link className={styles.link} to={story.href}>
                    Related guide
                  </Link>
                </div>
              </article>
            ))}
          </div>
        </section>
      </main>
    </Layout>
  );
}
