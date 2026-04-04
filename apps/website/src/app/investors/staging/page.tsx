import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { InvestorCtaButtons } from "@/components/investor-cta-buttons";

const toplineMetrics = [
  { label: "Pilot customers", value: "1", note: "Sigma One live" },
  { label: "Pipeline ACV", value: "$240K", note: "Active discussions" },
  { label: "Revenue streams", value: "4", note: "Subscription-led mix" },
  { label: "Infrastructure savings", value: "50-75%", note: "vs cloud list price" },
];

const pressureTimeline = [
  {
    title: "Then",
    subtitle: "Quarterly model releases",
    detail: "Teams could adapt stack decisions on a slow cadence.",
  },
  {
    title: "Now",
    subtitle: "Weekly model releases",
    detail: "Stack churn steals execution time and burns runway.",
  },
  {
    title: "Our position",
    subtitle: "One stable shipping loop",
    detail: "We absorb model/tool changes without forcing customer rewrites.",
  },
];

const deliveryLoop = [
  { step: "1", title: "Spec in", detail: "Customer writes a product spec in plain language." },
  { step: "2", title: "Agent orchestration", detail: "PM agent decomposes and assigns specialist agents." },
  { step: "3", title: "Quality gates", detail: "Review, tests, and security checks run on each PR." },
  { step: "4", title: "Deploy + heal", detail: "Deployment and monitoring run in the same operating system." },
];

const replacementMap = [
  { service: "5D Data", replaces: "AWS RDS", builtOn: "CloudNativePG" },
  { service: "5D Store", replaces: "S3", builtOn: "SeaweedFS" },
  { service: "5D Inference", replaces: "SageMaker", builtOn: "KubeAI" },
  { service: "5D Observe", replaces: "CloudWatch / Datadog", builtOn: "Prometheus + Grafana" },
  { service: "5D Deploy", replaces: "CI/CD SaaS", builtOn: "Argo CD" },
  { service: "5D Vault", replaces: "Secrets Manager", builtOn: "OpenBao" },
  { service: "5D Edge", replaces: "CloudFront / Route53", builtOn: "Cloudflare" },
];

const tractionProof = [
  { title: "Pilot in production", text: "Sigma One live pilot and partnership." },
  { title: "Pipeline validated", text: "Bloq in active discussion (~$240K ACV)." },
  { title: "Execution depth", text: "17+ bare-metal deployments completed." },
  { title: "System maturity", text: "22 coordinated agents built and operating." },
];

const fundUse = [
  { bucket: "Engineering hires", amount: "$300-400K", width: 100 },
  { bucket: "Founder salary", amount: "$100-120K", width: 32 },
  { bucket: "Market infra", amount: "$20-40K", width: 12 },
  { bucket: "Lab server", amount: "$16-20K", width: 8 },
  { bucket: "Model costs", amount: "$30-50K", width: 16 },
];

export default function InvestorsStagingPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      <div className="fixed inset-0 noise-overlay z-[3]" />

      <Header />

      <main className="relative z-10 pt-24">
        <section className="py-14 px-6">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-10">
              <span className="inline-flex items-center rounded-full border border-cyan/30 bg-cyan/10 px-3 py-1 text-xs tracking-wider text-cyan">
                Investor Deck - Staging Variant
              </span>
              <h1 className="text-4xl md:text-6xl font-bold mt-4 mb-4">
                5D Labs: <span className="gradient-text">Spec in, software out</span>
              </h1>
              <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
                Same thesis, lower cognitive load. This version compresses narrative into visual signals for faster partner-level review.
              </p>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {toplineMetrics.map((item) => (
                <article key={item.label} className="rounded-xl border border-border/50 bg-card/30 p-5 backdrop-blur-sm">
                  <p className="text-3xl font-bold gradient-text">{item.value}</p>
                  <p className="text-sm font-semibold mt-2">{item.label}</p>
                  <p className="text-xs text-muted-foreground mt-1">{item.note}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section className="py-14 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto grid lg:grid-cols-2 gap-6">
            <article className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
              <h2 className="text-2xl font-semibold mb-4">Why now</h2>
              <div className="space-y-3">
                {pressureTimeline.map((item) => (
                  <div key={item.title} className="rounded-lg border border-border/50 bg-background/40 p-4">
                    <p className="text-xs uppercase tracking-widest text-cyan">{item.title}</p>
                    <p className="text-base font-semibold mt-1">{item.subtitle}</p>
                    <p className="text-sm text-muted-foreground mt-1">{item.detail}</p>
                  </div>
                ))}
              </div>
            </article>

            <article className="rounded-xl border border-cyan/30 bg-cyan/5 p-6 backdrop-blur-sm">
              <h2 className="text-2xl font-semibold mb-4">How it works</h2>
              <div className="space-y-3">
                {deliveryLoop.map((item) => (
                  <div key={item.step} className="grid grid-cols-[36px_1fr] items-start gap-3">
                    <div className="size-9 rounded-full bg-cyan/20 border border-cyan/30 text-cyan font-semibold flex items-center justify-center">
                      {item.step}
                    </div>
                    <div>
                      <p className="font-semibold">{item.title}</p>
                      <p className="text-sm text-muted-foreground">{item.detail}</p>
                    </div>
                  </div>
                ))}
              </div>
            </article>
          </div>
        </section>

        <section className="py-14 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <h2 className="text-2xl font-semibold mb-4">Service replacement map</h2>
            <div className="overflow-x-auto rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
              <table className="w-full text-sm">
                <thead className="bg-background/50 text-left">
                  <tr>
                    <th className="px-4 py-3 font-semibold">5D service</th>
                    <th className="px-4 py-3 font-semibold">Replaces</th>
                    <th className="px-4 py-3 font-semibold">Built on</th>
                  </tr>
                </thead>
                <tbody>
                  {replacementMap.map((row) => (
                    <tr key={row.service} className="border-t border-border/40">
                      <td className="px-4 py-3 font-medium">{row.service}</td>
                      <td className="px-4 py-3 text-muted-foreground">{row.replaces}</td>
                      <td className="px-4 py-3 text-muted-foreground">{row.builtOn}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </section>

        <section className="py-14 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto grid lg:grid-cols-2 gap-6">
            <article className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
              <h2 className="text-2xl font-semibold mb-4">Traction proof</h2>
              <div className="grid sm:grid-cols-2 gap-3">
                {tractionProof.map((item) => (
                  <div key={item.title} className="rounded-lg border border-border/50 bg-background/40 p-4">
                    <p className="text-sm font-semibold">{item.title}</p>
                    <p className="text-sm text-muted-foreground mt-1">{item.text}</p>
                  </div>
                ))}
              </div>
            </article>

            <article className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
              <h2 className="text-2xl font-semibold mb-1">Use of funds</h2>
              <p className="text-sm text-muted-foreground mb-4">$750K target. 18 months to cash-flow positive.</p>
              <div className="space-y-3">
                {fundUse.map((line) => (
                  <div key={line.bucket}>
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span>{line.bucket}</span>
                      <span className="text-muted-foreground">{line.amount}</span>
                    </div>
                    <div className="h-2 w-full rounded-full bg-background/70 overflow-hidden">
                      <div className="h-full rounded-full bg-gradient-to-r from-cyan to-indigo-500" style={{ width: `${line.width}%` }} />
                    </div>
                  </div>
                ))}
              </div>
            </article>
          </div>
        </section>

        <section className="py-16 px-6 border-t border-border/30">
          <div className="max-w-3xl mx-auto text-center">
            <h2 className="text-3xl font-bold mb-3">Staging CTA</h2>
            <p className="text-muted-foreground mb-8">
              If this variant reads better, we can promote visual patterns back into the production deck.
            </p>
            <InvestorCtaButtons />
          </div>
        </section>

        <Footer />
      </main>
    </div>
  );
}
