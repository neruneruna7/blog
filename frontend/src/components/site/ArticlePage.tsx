import type { ArticleMeta, SiteConfig } from "@/lib/articles";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ArticleMetaList } from "@/components/site/ArticleMetaList";
import { TagList } from "@/components/site/TagList";

type ArticlePageProps = {
  site: SiteConfig;
  article: ArticleMeta;
  bodyHtml: string;
};

export function ArticlePage({ site, article, bodyHtml }: ArticlePageProps) {
  return (
    <article className="py-12 md:py-16">
      <header className="mx-auto max-w-4xl">
        <div className="mb-4 flex items-center gap-3">
          <Badge variant="secondary" className="font-decorative tracking-[0.2em]">
            Article
          </Badge>
        </div>

        <h1 className="font-heading text-4xl leading-tight tracking-tight text-balance md:text-5xl">
          {article.title}
        </h1>

        <p className="mt-5 max-w-2xl text-base leading-7 text-muted-foreground">
          {article.description}
        </p>

        <div className="mt-6">
          <ArticleMetaList site={site} article={article} />
        </div>

        {article.tags.length > 0 ? (
          <div className="mt-6">
            <TagList tags={article.tags} />
          </div>
        ) : null}
      </header>

      <Separator className="my-10" />

      <div
        className="article-body mx-auto"
        dangerouslySetInnerHTML={{ __html: bodyHtml }}
      />
    </article>
  );
}
