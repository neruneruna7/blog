import type { SiteConfig } from "@/lib/articles";

type SiteHeaderProps = {
  site: SiteConfig;
  currentPath?: string;
};

export function SiteHeader({ site, currentPath = "/" }: SiteHeaderProps) {
  const normalizedPath = currentPath.replace(/\/+$/, "") || "/";
  const isHome = normalizedPath === "/";
  const homeLinkClassName = [
    "inline-flex min-h-10 items-center border-b-2 px-3 text-sm font-medium tracking-wide no-underline transition-colors",
    isHome
      ? "border-primary text-foreground"
      : "border-transparent text-muted-foreground hover:border-border hover:text-foreground",
  ].join(" ");

  return (
    <header className="border-b border-border/70 bg-card/80">
      <div className="mx-auto grid w-[min(100%-2rem,64rem)] grid-cols-[minmax(0,max-content)_auto] items-center justify-start gap-4 py-4">
        <a href="/" className="min-w-0 no-underline">
          <span className="block truncate font-site text-xl font-semibold tracking-wide">
            {site.title}
          </span>
          <span className="block truncate text-sm text-muted-foreground">
            {site.description}
          </span>
        </a>

        <nav aria-label="メインナビゲーション" className="shrink-0">
          <a
            href="/"
            className={homeLinkClassName}
            aria-current={isHome ? "page" : undefined}
          >
            Home
          </a>
        </nav>
      </div>
    </header>
  );
}
