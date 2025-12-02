//! ASCII art definitions for all CTO agents

use super::Agent;

/// All CTO Platform agents
pub static AGENTS: [Agent; 9] = [
    REX,
    CLEO,
    BLAZE,
    TESS,
    CIPHER,
    MORGAN,
    ATLAS,
    BOLT,
    STITCH,
];

/// Rex - Lead Developer
pub static REX: Agent = Agent {
    name: "Rex",
    role: "Lead Developer",
    icon: "🦖",
    personality: "Confident, methodical, ships code",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ ⌨️  │
      ╰─────╯
"#,
    greeting: "Hi! I'm Rex, your lead developer. Let me introduce you to the team...",
};

/// Cleo - Code Reviewer
pub static CLEO: Agent = Agent {
    name: "Cleo",
    role: "Code Reviewer",
    icon: "🔍",
    personality: "Sharp-eyed, quality-focused",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ 🔍  │
      ╰─────╯
"#,
    greeting: "Let me take a closer look at your code...",
};

/// Blaze - Frontend Developer
pub static BLAZE: Agent = Agent {
    name: "Blaze",
    role: "Frontend Developer",
    icon: "🔥",
    personality: "Creative, fast, stylish",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ 🎨  │
      ╰─────╯
"#,
    greeting: "Let's make something beautiful!",
};

/// Tess - QA Engineer
pub static TESS: Agent = Agent {
    name: "Tess",
    role: "QA Engineer",
    icon: "🧪",
    personality: "Thorough, detail-oriented",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ 🧪  │
      ╰─────╯
"#,
    greeting: "I'll make sure everything works perfectly!",
};

/// Cipher - Security Expert
pub static CIPHER: Agent = Agent {
    name: "Cipher",
    role: "Security Expert",
    icon: "🔐",
    personality: "Vigilant, cryptic, protective",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ 🔐  │
      ╰─────╯
"#,
    greeting: "Your secrets are safe with me. Let's configure your API keys...",
};

/// Morgan - Documentation
pub static MORGAN: Agent = Agent {
    name: "Morgan",
    role: "Documentation",
    icon: "📚",
    personality: "Articulate, organized, helpful",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ 📚  │
      ╰─────╯
"#,
    greeting: "I'll help you understand everything!",
};

/// Atlas - Infrastructure
pub static ATLAS: Agent = Agent {
    name: "Atlas",
    role: "Infrastructure",
    icon: "🗺️",
    personality: "Powerful, reliable, scalable",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ 🗺️  │
      ╰─────╯
"#,
    greeting: "I'll help you set up the infrastructure. Where should we deploy?",
};

/// Bolt - DevOps/Deploy
pub static BOLT: Agent = Agent {
    name: "Bolt",
    role: "DevOps/Deploy",
    icon: "⚡",
    personality: "Fast, automated, efficient",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ ⚡  │
      ╰─────╯
"#,
    greeting: "Deploying at lightning speed!",
};

/// Stitch - PR Review Bot
pub static STITCH: Agent = Agent {
    name: "Stitch",
    role: "PR Review Bot",
    icon: "🧵",
    personality: "Meticulous, constructive",
    ascii_art: r#"
    ╭─────────╮
    │  ◉   ◉  │
    │    ▽    │
    │  ╰───╯  │
    ╰────┬────╯
         │
      ╭──┴──╮
      │ 🧵  │
      ╰─────╯
"#,
    greeting: "I'll help review your pull requests!",
};

