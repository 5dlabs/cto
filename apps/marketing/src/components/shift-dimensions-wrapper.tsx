export function ShiftDimensionsWrapper({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        minHeight: "100%",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {children}
    </div>
  );
}
