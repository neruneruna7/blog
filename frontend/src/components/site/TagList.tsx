import { Badge } from "@/components/ui/badge";

type TagListProps = {
  tags: string[];
};

export function TagList({ tags }: TagListProps) {
  return (
    <ul className="m-0 flex list-none flex-wrap gap-2 p-0" aria-label="Tags">
      {tags.map((tag) => (
        <li key={tag}>
          <Badge variant="outline">{tag}</Badge>
        </li>
      ))}
    </ul>
  );
}
