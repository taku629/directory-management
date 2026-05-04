export function LockedPanel({
  title,
  phase,
  blurb,
  previewItems,
}: {
  title: string;
  phase: string;
  blurb: string;
  previewItems: string[];
}) {
  return (
    <div className="locked">
      <div className="lock-icon">🔒</div>
      <div style={{ fontSize: 18, fontWeight: 600, marginBottom: 4 }}>
        {title}
      </div>
      <div style={{ fontSize: 11, color: "var(--accent)", marginBottom: 12 }}>
        {phase}
      </div>
      <div style={{ maxWidth: 480, margin: "0 auto 16px", lineHeight: 1.5 }}>
        {blurb}
      </div>
      <ul
        style={{
          textAlign: "left",
          maxWidth: 460,
          margin: "0 auto",
          paddingLeft: 18,
          color: "var(--fg-dim)",
          fontSize: 13,
        }}
      >
        {previewItems.map((item, i) => (
          <li key={i} style={{ marginBottom: 4 }}>
            {item}
          </li>
        ))}
      </ul>
    </div>
  );
}
