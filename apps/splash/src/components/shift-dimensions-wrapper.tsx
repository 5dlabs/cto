export function ShiftDimensionsWrapper({ children }: { children: React.ReactNode }) {
  return (
    <div
      className="shift-dimensions"
      style={{
        minHeight: "100%",
        display: "flex",
        flexDirection: "column",
        transformOrigin: "50% 50%",
        transformStyle: "flat",
      }}
    >
      {children}
    </div>
  );
}
