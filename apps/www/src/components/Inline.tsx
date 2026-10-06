import type { ReactNode } from "react";
import { Link } from "react-router-dom";

const TOKEN = /\[([^\]]+)\]\(([^)\s]+)\)|\*\*([^*]+)\*\*|\*([^*]+)\*|`([^`]+)`/g;

/** Renders the content markup: *italic*, **bold**, `code`, [label](href). */
export function Inline({ text }: { text: string }) {
  return <>{parse(text, "i")}</>;
}

function parse(text: string, prefix: string): ReactNode[] {
  const out: ReactNode[] = [];
  let last = 0;
  let index = 0;
  for (const match of text.matchAll(TOKEN)) {
    const start = match.index ?? 0;
    if (start > last) out.push(text.slice(last, start));
    const key = `${prefix}-${index++}`;
    const [, label, href, bold, italic, code] = match;
    if (label !== undefined && href !== undefined) {
      out.push(<SmartLink key={key} href={href}>{parse(label, key)}</SmartLink>);
    } else if (bold !== undefined) {
      out.push(<strong key={key}>{parse(bold, key)}</strong>);
    } else if (italic !== undefined) {
      out.push(<em key={key}>{parse(italic, key)}</em>);
    } else if (code !== undefined) {
      out.push(<code key={key}>{code}</code>);
    }
    last = start + match[0].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

export function SmartLink({
  href,
  children,
  className,
}: {
  href: string;
  children: ReactNode;
  className?: string;
}) {
  if (href.startsWith("/") && !href.startsWith("//") && !/\.(txt|xml|svg|png)$/.test(href)) {
    return (
      <Link to={href} className={className}>
        {children}
      </Link>
    );
  }
  const external = /^https?:/.test(href);
  return (
    <a
      href={href}
      className={className}
      {...(external ? { target: "_blank", rel: "noopener noreferrer" } : {})}
    >
      {children}
    </a>
  );
}

/** Plain text, for attributes and structured data. */
export function plainText(text: string): string {
  return text
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/`([^`]+)`/g, "$1");
}
