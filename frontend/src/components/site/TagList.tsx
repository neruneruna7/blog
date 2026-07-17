type TagListProps = {
  tags: string[];
};

export function TagList({ tags }: TagListProps) {
  return (
    <ul className="m-0 flex list-none flex-wrap gap-2 p-0" aria-label="Tags">
      {tags.map((tag) => (
        <li key={tag}>
          <span className="inline-flex h-5 w-fit shrink-0 items-center justify-center border border-border px-2 py-0.5 text-xs font-medium whitespace-nowrap text-foreground transform-[skewX(-12deg)]">
            <span className="inline-block transform-[skewX(12deg)]">{tag}</span>
          </span>
        </li>
      ))}
    </ul>
  );
}
