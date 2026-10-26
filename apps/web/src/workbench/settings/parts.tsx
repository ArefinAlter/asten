import type { ReactNode } from "react";
import { Icon } from "../icons";

export function Head({
  title,
  children,
}: {
  title: string;
  children?: ReactNode;
}) {
  return (
    <header className="settings-head">
      <h1>{title}</h1>
      {children && <p className="muted">{children}</p>}
    </header>
  );
}

export function Card({
  title,
  children,
}: {
  title?: string;
  children: ReactNode;
}) {
  return (
    <section className="wb-card settings-card">
      {title && <h2>{title}</h2>}
      <div className="wb-rows">{children}</div>
    </section>
  );
}

export function SettingRow({
  title,
  note,
  children,
}: {
  title: string;
  note?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <div className="setting-row">
      <div className="setting-text">
        <strong>{title}</strong>
        {note && <small>{note}</small>}
      </div>
      <div className="setting-control">{children}</div>
    </div>
  );
}

export function Fixed({ title, note }: { title: string; note: string }) {
  return (
    <SettingRow title={title} note={note}>
      <span className="wb-pill">
        <Icon name="lock" size={12} /> fixed
      </span>
    </SettingRow>
  );
}

export function Switch({
  value,
  onChange,
  label,
  disabled,
}: {
  value: boolean;
  onChange: (v: boolean) => void;
  label: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={value}
      aria-label={label}
      disabled={disabled}
      className={`wb-switch${value ? " on" : ""}`}
      onClick={() => onChange(!value)}
    >
      <span />
    </button>
  );
}
