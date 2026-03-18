declare namespace JSX {
  interface IntrinsicElements {
    "lemon-slice-widget": React.DetailedHTMLProps<
      React.HTMLAttributes<HTMLElement> & {
        "agent-id"?: string;
        "initial-state"?: "active" | "minimized";
      },
      HTMLElement
    >;
  }
}
