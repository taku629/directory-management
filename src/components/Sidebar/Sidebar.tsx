import { useEffect, useState } from "react";
import clsx from "clsx";
import { useAppStore } from "../../lib/store";
import {
  addFavorite,
  listFavorites,
  listSmartFolders,
  removeFavorite,
} from "../../lib/tauri";
import type { Favorite, SmartFolder, Tag } from "../../lib/types";
import { open } from "@tauri-apps/plugin-dialog";
import { PRESETS } from "../../lib/presets";
import type { PresetId } from "../../lib/store";

export function Sidebar() {
  const view = useAppStore((s) => s.view);
  const setView = useAppStore((s) => s.setView);
  const tags = useAppStore((s) => s.tags);
  const [favorites, setFavorites] = useState<Favorite[]>([]);
  const [smarts, setSmarts] = useState<SmartFolder[]>([]);

  useEffect(() => {
    listFavorites().then(setFavorites).catch(console.error);
    listSmartFolders().then(setSmarts).catch(console.error);
  }, [view]);

  async function addFav() {
    const picked = await open({ directory: true });
    if (typeof picked === "string") {
      const name = picked.split(/[\\/]/).pop() ?? picked;
      await addFavorite(picked, name);
      setFavorites(await listFavorites());
    }
  }

  return (
    <nav className="sidebar">
      <h3>Library</h3>
      <NavItem
        icon="🏠"
        label="Home"
        active={view.kind === "browse" && view.path === ""}
        onClick={() => setView({ kind: "browse", path: "" })}
      />
      <NavItem
        icon="🔍"
        label="All Files"
        active={view.kind === "search"}
        onClick={() => setView({ kind: "search", text: "" })}
      />

      <h3>
        Favorites
        <button
          style={{ marginLeft: 8, padding: "0 6px" }}
          onClick={addFav}
          title="Add favorite folder"
        >
          +
        </button>
      </h3>
      {favorites.length === 0 && (
        <div className="nav-item" style={{ color: "var(--fg-dim)" }}>
          (empty)
        </div>
      )}
      {favorites.map((f) => (
        <FavoriteItem
          key={f.id}
          fav={f}
          active={view.kind === "browse" && view.path === f.path}
          onClick={() => setView({ kind: "browse", path: f.path })}
          onRemove={async () => {
            await removeFavorite(f.id);
            setFavorites(await listFavorites());
          }}
        />
      ))}

      <h3>Built-in</h3>
      {(Object.entries(PRESETS) as [PresetId, (typeof PRESETS)[PresetId]][]).map(
        ([id, p]) => (
          <NavItem
            key={id}
            icon={p.icon}
            label={p.label}
            active={view.kind === "preset" && view.preset === id}
            onClick={() => setView({ kind: "preset", preset: id })}
          />
        ),
      )}

      <h3>Smart Folders</h3>
      {smarts.length === 0 && (
        <div className="nav-item" style={{ color: "var(--fg-dim)" }}>
          (none yet)
        </div>
      )}
      {smarts.map((s) => (
        <NavItem
          key={s.id}
          icon={s.icon ?? "✨"}
          label={s.name}
          active={view.kind === "smart" && view.smartFolderId === s.id}
          onClick={() => setView({ kind: "smart", smartFolderId: s.id })}
        />
      ))}

      <h3>Tags</h3>
      {tags.length === 0 && (
        <div className="nav-item" style={{ color: "var(--fg-dim)" }}>
          (no tags)
        </div>
      )}
      {tags.map((t: Tag) => (
        <NavItem
          key={t.id}
          icon={
            <span
              className="dot"
              style={{
                display: "inline-block",
                width: 10,
                height: 10,
                borderRadius: 5,
                background: t.color ?? "var(--fg-dim)",
              }}
            />
          }
          label={t.name}
          active={view.kind === "tag" && view.tagId === t.id}
          onClick={() => setView({ kind: "tag", tagId: t.id })}
        />
      ))}

      <h3>Tools</h3>
      <NavItem
        icon="📊"
        label="Disk Usage"
        active={view.kind === "disk-usage"}
        onClick={() => setView({ kind: "disk-usage" })}
      />
      <NavItem
        icon="🔁"
        label="Duplicates"
        active={view.kind === "duplicates"}
        onClick={() => setView({ kind: "duplicates" })}
      />
      <NavItem
        icon="⚙"
        label="Rules"
        active={view.kind === "rules"}
        onClick={() => setView({ kind: "rules" })}
      />
      <NavItem
        icon="✦"
        label="AI"
        active={view.kind === "ai"}
        onClick={() => setView({ kind: "ai" })}
      />
    </nav>
  );
}

function NavItem({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  onClick?: () => void;
}) {
  return (
    <div className={clsx("nav-item", { active })} onClick={onClick}>
      <span className="icon">{icon}</span>
      <span>{label}</span>
    </div>
  );
}

function FavoriteItem({
  fav,
  active,
  onClick,
  onRemove,
}: {
  fav: Favorite;
  active: boolean;
  onClick: () => void;
  onRemove: () => void;
}) {
  return (
    <div
      className={clsx("nav-item", { active })}
      onClick={onClick}
      onContextMenu={(e) => {
        e.preventDefault();
        if (confirm(`Remove ${fav.name ?? fav.path}?`)) onRemove();
      }}
      title={fav.path}
    >
      <span className="icon">📁</span>
      <span>{fav.name ?? fav.path}</span>
    </div>
  );
}
