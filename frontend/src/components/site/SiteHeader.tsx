import type { SiteConfig } from "@/lib/articles";

type SiteHeaderProps = {
  site: SiteConfig;
};

export function SiteHeader({ site }: SiteHeaderProps) {
  return (
    <header className="border-b border-border/70 bg-card/80">
      <div className="mx-auto flex w-[min(100%-2rem,64rem)] flex-col gap-1 py-5">
        <a
          href="/"
          className="w-fit font-site text-xl font-semibold tracking-wide no-underline"
        >
          {site.title}
          <p className="m-0 text-sm text-muted-foreground">
            {site.description}
          </p>
        </a>
      </div>
    </header>
  );
}
