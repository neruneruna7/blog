import type { ArticleMeta, SiteConfig } from "@/lib/articles";

type ArticleMetaListProps = {
  site: SiteConfig;
  article: ArticleMeta;
};

export function ArticleMetaList({ site, article }: ArticleMetaListProps) {
  const dateFormatter = new Intl.DateTimeFormat(site.language);
  const publishedAt = dateFormatter.format(new Date(article.publishedAt));
  const updatedAt = article.updatedAt
    ? dateFormatter.format(new Date(article.updatedAt))
    : null;

  return (
    <dl className="flex flex-wrap gap-x-8 gap-y-4 text-sm">
      <div className="grid gap-1">
        <dt className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
          Published
        </dt>
        <dd className="m-0">
          <time dateTime={article.publishedAt}>{publishedAt}</time>
        </dd>
      </div>

      {article.updatedAt && updatedAt ? (
        <div className="grid gap-1">
          <dt className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            Updated
          </dt>
          <dd className="m-0">
            <time dateTime={article.updatedAt}>{updatedAt}</time>
          </dd>
        </div>
      ) : null}
    </dl>
  );
}
