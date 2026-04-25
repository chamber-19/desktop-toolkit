/**
 * ReleaseNotes.jsx — Renders markdown release notes safely.
 *
 * Uses `marked` to convert markdown to HTML and `DOMPurify` to sanitize
 * the output before injecting it via dangerouslySetInnerHTML. Only basic
 * markdown is enabled (headings, bold/italic, lists, links, code,
 * blockquotes) — extended features (tables, task lists, footnotes) are
 * intentionally disabled.
 *
 * Props:
 *   notes         {string|null|undefined} — raw markdown string
 *   fallbackUrl   {string|null}           — GitHub releases URL shown when
 *                                           notes are absent (optional)
 *   className     {string}                — additional CSS class (optional)
 */

import { useMemo } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";
import "./ReleaseNotes.css";

// Configure marked: disable features not needed for release notes.
const MARKED_OPTS = {
  gfm: true,         // GitHub-Flavored Markdown (fenced code, autolinks)
  breaks: false,     // Keep hard-wrap semantics off — release notes use blank lines
  pedantic: false,
};

// DOMPurify allowed tags/attrs — enough for headings, lists, links, code,
// blockquotes, emphasis. Tables, iframes, scripts, etc. are stripped.
const PURIFY_CONFIG = {
  ALLOWED_TAGS: [
    "h1", "h2", "h3", "h4", "h5", "h6",
    "p", "br",
    "strong", "em", "s", "del",
    "ul", "ol", "li",
    "a",
    "code", "pre",
    "blockquote",
    "hr",
  ],
  ALLOWED_ATTR: ["href", "title", "target", "rel"],
};

/**
 * Parse notes string → sanitized HTML string.
 * Returns null when input is falsy or parsing fails.
 */
function parseNotes(notes) {
  if (!notes || typeof notes !== "string") return null;
  const trimmed = notes.trim();
  if (!trimmed) return null;
  try {
    const raw = marked.parse(trimmed, MARKED_OPTS);
    const clean = DOMPurify.sanitize(raw, PURIFY_CONFIG);
    return clean || null;
  } catch {
    return null;
  }
}

export function ReleaseNotes({ notes, fallbackUrl = null, className = "" }) {
  const html = useMemo(() => parseNotes(notes), [notes]);

  if (html) {
    return (
      <div
        className={`release-notes${className ? ` ${className}` : ""}`}
        // DOMPurify-sanitized output — safe to inject.
        // eslint-disable-next-line react/no-danger
        dangerouslySetInnerHTML={{ __html: html }}
      />
    );
  }

  if (fallbackUrl) {
    return (
      <p className={`release-notes-fallback${className ? ` ${className}` : ""}`}>
        See{" "}
        <a
          href={fallbackUrl}
          target="_blank"
          rel="noreferrer"
          className="release-notes-fallback-link"
        >
          release notes on GitHub
        </a>
        .
      </p>
    );
  }

  return null;
}

export default ReleaseNotes;
