// Structured condition/action editor — replaces the JSON textareas with
// type-aware form rows. Round-trips back to the same JSON the backend expects.

import { useState } from "react";
import { listTags } from "../../lib/tauri";
import type { Tag } from "../../lib/types";

export type Condition =
  | { type: "NameMatches"; value: string }
  | { type: "ExtensionIn"; value: string[] }
  | { type: "MimeStartsWith"; value: string }
  | { type: "SizeBetween"; value: { min?: number; max?: number } }
  | { type: "ModifiedWithinDays"; value: number }
  | { type: "PathIsIn"; value: string }
  | { type: "HasTag"; value: number };

export type Action =
  | { type: "MoveTo"; value: string }
  | { type: "RenameTo"; value: string }
  | { type: "AddTag"; value: number }
  | { type: "SetColor"; value: string }
  | { type: "SetRating"; value: number }
  | { type: "Notify"; value: string };

const COND_TYPES: Condition["type"][] = [
  "NameMatches",
  "ExtensionIn",
  "MimeStartsWith",
  "SizeBetween",
  "ModifiedWithinDays",
  "PathIsIn",
  "HasTag",
];
const ACTION_TYPES: Action["type"][] = [
  "MoveTo",
  "RenameTo",
  "AddTag",
  "SetColor",
  "SetRating",
  "Notify",
];

const COND_DEFAULTS: Record<Condition["type"], unknown> = {
  NameMatches: "*.pdf",
  ExtensionIn: ["pdf"],
  MimeStartsWith: "image/",
  SizeBetween: { min: 1024 },
  ModifiedWithinDays: 7,
  PathIsIn: "",
  HasTag: 0,
};
const ACTION_DEFAULTS: Record<Action["type"], unknown> = {
  MoveTo: "/path/to/{year}/{month}",
  RenameTo: "{stem}_v2.{ext}",
  AddTag: 0,
  SetColor: "#5b9dff",
  SetRating: 3,
  Notify: "",
};

function defaultCondition(t: Condition["type"]): Condition {
  return { type: t, value: COND_DEFAULTS[t] } as Condition;
}
function defaultAction(t: Action["type"]): Action {
  return { type: t, value: ACTION_DEFAULTS[t] } as Action;
}

export function ConditionEditor({
  conditions,
  onChange,
  tags,
}: {
  conditions: Condition[];
  onChange: (c: Condition[]) => void;
  tags: Tag[];
}) {
  return (
    <div>
      {conditions.map((c, i) => (
        <ConditionRow
          key={i}
          c={c}
          tags={tags}
          onChange={(v) => {
            const next = [...conditions];
            next[i] = v;
            onChange(next);
          }}
          onRemove={() => onChange(conditions.filter((_, j) => j !== i))}
        />
      ))}
      <select
        value=""
        onChange={(e) => {
          const t = e.target.value as Condition["type"];
          if (t) onChange([...conditions, defaultCondition(t)]);
          e.target.value = "";
        }}
        style={{ marginTop: 4 }}
      >
        <option value="">+ Add condition…</option>
        {COND_TYPES.map((t) => (
          <option key={t} value={t}>
            {t}
          </option>
        ))}
      </select>
    </div>
  );
}

function ConditionRow({
  c,
  tags,
  onChange,
  onRemove,
}: {
  c: Condition;
  tags: Tag[];
  onChange: (c: Condition) => void;
  onRemove: () => void;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        padding: "4px 0",
      }}
    >
      <select
        value={c.type}
        onChange={(e) =>
          onChange(defaultCondition(e.target.value as Condition["type"]))
        }
        style={{ width: 150 }}
      >
        {COND_TYPES.map((t) => (
          <option key={t} value={t}>
            {t}
          </option>
        ))}
      </select>
      <ConditionValue c={c} tags={tags} onChange={onChange} />
      <button onClick={onRemove}>×</button>
    </div>
  );
}

function ConditionValue({
  c,
  tags,
  onChange,
}: {
  c: Condition;
  tags: Tag[];
  onChange: (c: Condition) => void;
}) {
  switch (c.type) {
    case "NameMatches":
    case "MimeStartsWith":
    case "PathIsIn":
      return (
        <input
          value={c.value}
          onChange={(e) =>
            onChange({ ...c, value: e.target.value } as Condition)
          }
          style={{ flex: 1 }}
          placeholder={
            c.type === "NameMatches"
              ? "*.pdf"
              : c.type === "MimeStartsWith"
                ? "image/"
                : "/path/prefix"
          }
        />
      );
    case "ExtensionIn":
      return (
        <input
          value={c.value.join(",")}
          onChange={(e) =>
            onChange({
              ...c,
              value: e.target.value
                .split(",")
                .map((s) => s.trim().replace(/^\./, ""))
                .filter(Boolean),
            })
          }
          style={{ flex: 1 }}
          placeholder="pdf,docx,md"
        />
      );
    case "SizeBetween":
      return (
        <>
          <input
            type="number"
            placeholder="min bytes"
            value={c.value.min ?? ""}
            onChange={(e) =>
              onChange({
                ...c,
                value: {
                  ...c.value,
                  min: e.target.value ? parseInt(e.target.value, 10) : undefined,
                },
              })
            }
            style={{ width: 110 }}
          />
          <input
            type="number"
            placeholder="max bytes"
            value={c.value.max ?? ""}
            onChange={(e) =>
              onChange({
                ...c,
                value: {
                  ...c.value,
                  max: e.target.value ? parseInt(e.target.value, 10) : undefined,
                },
              })
            }
            style={{ width: 110 }}
          />
        </>
      );
    case "ModifiedWithinDays":
      return (
        <input
          type="number"
          value={c.value}
          onChange={(e) =>
            onChange({ ...c, value: parseInt(e.target.value || "0", 10) })
          }
          style={{ width: 100 }}
        />
      );
    case "HasTag":
      return (
        <select
          value={c.value}
          onChange={(e) =>
            onChange({ ...c, value: parseInt(e.target.value, 10) })
          }
          style={{ flex: 1 }}
        >
          <option value={0}>(select tag)</option>
          {tags.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>
      );
  }
}

export function ActionEditor({
  actions,
  onChange,
  tags,
}: {
  actions: Action[];
  onChange: (a: Action[]) => void;
  tags: Tag[];
}) {
  return (
    <div>
      {actions.map((a, i) => (
        <ActionRow
          key={i}
          a={a}
          tags={tags}
          onChange={(v) => {
            const next = [...actions];
            next[i] = v;
            onChange(next);
          }}
          onRemove={() => onChange(actions.filter((_, j) => j !== i))}
        />
      ))}
      <select
        value=""
        onChange={(e) => {
          const t = e.target.value as Action["type"];
          if (t) onChange([...actions, defaultAction(t)]);
          e.target.value = "";
        }}
        style={{ marginTop: 4 }}
      >
        <option value="">+ Add action…</option>
        {ACTION_TYPES.map((t) => (
          <option key={t} value={t}>
            {t}
          </option>
        ))}
      </select>
    </div>
  );
}

function ActionRow({
  a,
  tags,
  onChange,
  onRemove,
}: {
  a: Action;
  tags: Tag[];
  onChange: (a: Action) => void;
  onRemove: () => void;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 6,
        padding: "4px 0",
      }}
    >
      <select
        value={a.type}
        onChange={(e) =>
          onChange(defaultAction(e.target.value as Action["type"]))
        }
        style={{ width: 130 }}
      >
        {ACTION_TYPES.map((t) => (
          <option key={t} value={t}>
            {t}
          </option>
        ))}
      </select>
      <ActionValue a={a} tags={tags} onChange={onChange} />
      <button onClick={onRemove}>×</button>
    </div>
  );
}

function ActionValue({
  a,
  tags,
  onChange,
}: {
  a: Action;
  tags: Tag[];
  onChange: (a: Action) => void;
}) {
  switch (a.type) {
    case "MoveTo":
    case "RenameTo":
    case "Notify":
      return (
        <input
          value={a.value}
          onChange={(e) =>
            onChange({ ...a, value: e.target.value } as Action)
          }
          style={{ flex: 1 }}
          placeholder={
            a.type === "MoveTo"
              ? "/dest/{year}/{month}"
              : a.type === "RenameTo"
                ? "{stem}_archived.{ext}"
                : "message"
          }
        />
      );
    case "AddTag":
      return (
        <select
          value={a.value}
          onChange={(e) =>
            onChange({ ...a, value: parseInt(e.target.value, 10) })
          }
          style={{ flex: 1 }}
        >
          <option value={0}>(select tag)</option>
          {tags.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>
      );
    case "SetColor":
      return (
        <input
          type="color"
          value={a.value}
          onChange={(e) => onChange({ ...a, value: e.target.value })}
        />
      );
    case "SetRating":
      return (
        <select
          value={a.value}
          onChange={(e) =>
            onChange({ ...a, value: parseInt(e.target.value, 10) })
          }
        >
          {[0, 1, 2, 3, 4, 5].map((n) => (
            <option key={n} value={n}>
              {n} ★
            </option>
          ))}
        </select>
      );
  }
}

// Convenience hook: load tags once for both editors.
export function useTagList() {
  const [tags, setTags] = useState<Tag[]>([]);
  useState(() => {
    listTags().then(setTags).catch(console.error);
  });
  return tags;
}
