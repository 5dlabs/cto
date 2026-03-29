import type { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { Header } from "@/components/cto/header";
import { LemonSliceWidget } from "@/components/cto/lemon-slice-widget";

export const metadata: Metadata = {
  title: "Morgan — Your AI Business Coordinator | Talk via Chat, Voice, or Video",
  description:
    "Morgan is your single point of contact for the CTO platform. Chat, voice, or video—from any device. Your control agent coordinates the whole team.",
  openGraph: {
    title: "Morgan — Your AI Business Coordinator | Talk via Chat, Voice, or Video",
    description:
      "Morgan is your single point of contact. Chat, voice, or video—from any device. Your control agent coordinates the whole team.",
    url: "https://5dlabs.ai/cto/morgan",
  },
};

const interactionMethods = [
  {
    title: "Instant messaging",
    description:
      "Reach Morgan from Telegram, Discord, Slack, WhatsApp, iMessage, Signal, Feishu, Google Chat, Microsoft Teams, LINE, Matrix, Mattermost, IRC, Nostr, WebChat, and more.",
  },
  {
    title: "Desktop app",
    description:
      "A native desktop client that keeps Morgan one click away. No browser tab required.",
  },
  {
    title: "Mobile app",
    description:
      "Take Morgan with you. Full voice and chat on iOS and Android.",
  },
  {
    title: "AR glasses (Even G2)",
    description:
      "Stay productive without being tethered to your computer. Talk to Morgan hands-free with display smart glasses.",
  },
  {
    title: "Meta Ray-Ban Display",
    description:
      "Talk to Morgan on WhatsApp hands-free — same conversation, your glasses.",
  },
  {
    title: "Rokid Glasses",
    description:
      "Same Morgan on your channels — voice, chat, or messages from your phone with Rokid on.",
  },
  {
    title: "Vuzix Z100",
    description:
      "Enterprise-grade smart glasses — Morgan on Slack, Teams, WhatsApp, or whatever you already use.",
  },
];

export default function MorganPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-0" />
      <div className="fixed inset-0 noise-overlay z-0" />

      <Header />

      <main className="relative z-10">
        {/* Hero */}
        <section className="min-h-[85vh] flex flex-col items-center justify-center px-6 py-24 pt-28">
          <div className="max-w-6xl mx-auto w-full grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-16 items-center">
            <div>
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-6">
                <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
                <span className="text-sm text-cyan font-medium">
                  Your single point of contact
                </span>
              </div>
              <h1 className="text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight mb-6">
                <span className="gradient-text glow-text-cyan">Meet Morgan</span>
              </h1>
              <p className="text-xl text-muted-foreground mb-8 max-w-xl">
                Your control agent. Chat, voice, or video—from any device. You
                only talk to Morgan; Morgan coordinates the rest.
              </p>
              <Link
                href="#talk"
                className="inline-flex items-center gap-2 px-6 py-3 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold hover:from-cyan-600 hover:to-blue-600 transition-all"
              >
                Talk to Morgan
              </Link>
            </div>
            <div className="relative aspect-square max-w-md mx-auto lg:mx-0">
              <Image
                src="/agents/morgan-hero.png?v=20260318"
                alt="Morgan — golden retriever cyberpunk avatar"
                fill
                className="object-contain"
                priority
                sizes="(max-width: 1024px) 100vw, 50vw"
              />
            </div>
          </div>
        </section>

        {/* How it works + Widget */}
        <section id="talk" className="py-20 px-6 scroll-mt-24">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                How <span className="gradient-text">it works</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Morgan is your single point of contact for the CTO platform.
                Talk to Morgan from anywhere—chat, voice, or video. Morgan
                coordinates the rest of the team.
              </p>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-start">
              <div className="space-y-8">
                {interactionMethods.map((method, i) => (
                  <div key={i} className="space-y-2">
                    <h3 className="text-lg font-semibold">
                      {method.title}
                    </h3>
                    <p className="text-muted-foreground">
                      {method.description}
                    </p>
                  </div>
                ))}
              </div>
              <div className="lg:sticky lg:top-28 flex flex-col items-center">
                <div className="rounded-xl border border-border bg-card/50 p-4 w-full max-w-[420px] min-h-[680px] sm:min-h-[720px] flex items-center justify-center">
                  <LemonSliceWidget
                    initialState="active"
                    inline
                    customActiveWidth={368}
                    customActiveHeight={560}
                    customMinimizedWidth={120}
                    customMinimizedHeight={187}
                    className="w-full h-full flex items-center justify-center"
                  />
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Even G2 Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Talk to Morgan on <span className="gradient-text">Even G2</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Stay productive without being tethered to your computer. Even G2
                display smart glasses let you talk to Morgan hands-free—anywhere
                you go.
              </p>
            </div>

            {/* Product shot: glasses + ring — establishes the hardware */}
            <div className="relative aspect-[16/9] max-w-2xl mx-auto rounded-xl overflow-hidden border border-border bg-card/30 mb-8">
              <Image
                src="/even-realities/g2-product-glasses-ring.png"
                alt="Even G2 smart glasses and companion ring"
                fill
                className="object-cover"
                sizes="(max-width: 768px) 100vw, 672px"
              />
            </div>

            <p className="mt-8 text-center text-sm text-muted-foreground">
              Work should be fun. It shouldn&apos;t be gruelling. Even G2 helps
              you stay in the flow without being chained to your desk.
            </p>

            <div className="mt-8 text-center">
              <a
                href="https://evenrealities.com/smart-glasses"
                target="_blank"
                rel="noopener noreferrer"
                className="text-cyan hover:text-cyan-400 transition-colors"
              >
                Learn more about Even G2 →
              </a>
            </div>
          </div>
        </section>

        {/* Meta Ray-Ban Display */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Talk to Morgan on{" "}
                <span className="gradient-text">Meta Ray-Ban Display</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                WhatsApp on your glasses. Same Morgan.
              </p>
            </div>

            <div className="relative aspect-[16/9] max-w-2xl mx-auto rounded-xl overflow-hidden border border-border bg-black mb-8">
              <Image
                src="/morgan/meta-ray-ban-display.png"
                alt="Ray-Ban Meta Display smart glasses and neural band"
                fill
                className="object-cover"
                sizes="(max-width: 768px) 100vw, 672px"
              />
            </div>
          </div>
        </section>

        {/* Rokid */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Talk to Morgan on{" "}
                <span className="gradient-text">Rokid Glasses</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Same Morgan. Your channels, your phone, your glasses.
              </p>
            </div>

            <div className="relative aspect-[16/9] max-w-2xl mx-auto rounded-xl overflow-hidden border border-border bg-black mb-8">
              <Image
                src="/morgan/rokid-glasses.png"
                alt="Rokid smart glasses in charging case"
                fill
                className="object-cover"
                sizes="(max-width: 768px) 100vw, 672px"
              />
            </div>
          </div>
        </section>

        {/* Vuzix Z100 */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Talk to Morgan on{" "}
                <span className="gradient-text">Vuzix Z100</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Same Morgan. Slack, Teams, WhatsApp — wherever you already work.
              </p>
            </div>

            <div className="relative aspect-[16/9] max-w-2xl mx-auto rounded-xl overflow-hidden border border-border bg-white mb-8">
              <Image
                src="/morgan/vuzix-z100.png"
                alt="Vuzix Z100 smart glasses"
                fill
                className="object-contain p-4"
                sizes="(max-width: 768px) 100vw, 672px"
              />
            </div>
          </div>
        </section>

        {/* Philosophy */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-3xl mx-auto text-center">
            <h2 className="text-2xl sm:text-3xl font-bold mb-6">
              One agent. One conversation.
            </h2>
            <p className="text-lg text-muted-foreground leading-relaxed">
              You don&apos;t need to manage a dozen tools or remember which agent
              does what. Morgan is your single point of contact. Chat, voice, or
              video—from anywhere. Morgan coordinates the rest.
            </p>
          </div>
        </section>
      </main>
    </div>
  );
}
