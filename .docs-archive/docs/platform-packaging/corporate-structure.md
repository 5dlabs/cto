# Corporate Structure Options for VC Funding

## Disclaimer

**This document is for informational purposes only and does not constitute legal, 
tax, or financial advice.** Consult with a qualified attorney and accountant 
before making corporate structure decisions. Laws change frequently, and individual 
circumstances vary significantly.

---

## Executive Summary

For Canadian founders seeking US VC funding, the **Delaware C-Corp** is typically 
the preferred structure. However, there are several options to consider, each with 
trade-offs around taxes, complexity, and investor preferences.

**Quick Recommendation:**
- Seeking US VC ($1M+): Delaware C-Corp
- Bootstrapping/Small rounds: Canadian Federal Corporation may suffice
- Hybrid: Canadian OpCo + US HoldCo (more complex, but tax advantages)

---

## Why Delaware C-Corp?

### The Standard for US VC

**95%+ of VC-backed startups** are Delaware C-Corps. Here's why:

```
┌─────────────────────────────────────────────────────────────────┐
│                Why VCs Prefer Delaware C-Corps                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Legal Familiarity                                               │
│  ─────────────────                                               │
│  • Lawyers know Delaware corporate law by heart                 │
│  • Faster, cheaper legal work                                   │
│  • Precedent for every situation                                │
│  • Court of Chancery specializes in corporate disputes          │
│                                                                  │
│  Standard Documents                                              │
│  ──────────────────                                              │
│  • SAFE, Series A docs are template-based                       │
│  • No "translation" for foreign structures                      │
│  • Easier due diligence                                         │
│  • Faster closes                                                │
│                                                                  │
│  Investor Expectations                                           │
│  ─────────────────────                                           │
│  • US VCs have US LPs (limited partners)                        │
│  • LPs expect US investments                                    │
│  • Some funds can't invest in foreign entities                  │
│  • Tax treatment is understood                                  │
│                                                                  │
│  Exit Mechanics                                                  │
│  ──────────────────                                              │
│  • US acquirers prefer US targets                               │
│  • IPO path is clearer                                          │
│  • Stock options work as expected                               │
│  • M&A lawyers don't need to learn new jurisdiction             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Delaware Specifics

**Why Delaware (not Nevada, Wyoming, etc.):**

| Factor | Delaware | Other States |
|--------|----------|--------------|
| Court of Chancery | Yes - expert business court | No |
| Case law depth | 200+ years | Limited |
| VC familiarity | Universal | "Why not Delaware?" |
| Privacy | Moderate | Some offer more |
| Franchise tax | Moderate | Some lower |

**Bottom line**: Non-Delaware corps raise friction with VCs. Unless there's a 
strong reason, default to Delaware.

---

## Options for Canadian Founders

### Option 1: Delaware C-Corp (US Only) ✅ Recommended for VC

**Structure:**
```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│                    Delaware C-Corp                               │
│                    (5D Labs, Inc.)                               │
│                         │                                        │
│              ┌──────────┴──────────┐                            │
│              │                     │                            │
│         Founder(s)            VC Investors                      │
│         (Canada)              (US)                               │
│                                                                  │
│  • All IP held by US corp                                       │
│  • Founder employed by US corp                                  │
│  • Can hire in US or Canada                                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Process:**
1. Incorporate in Delaware (online, ~$500)
2. Use registered agent service (~$100-300/year)
3. Get EIN (tax ID) from IRS
4. Open US bank account (Mercury, Brex, or similar)
5. File for ITIN (if needed for personal US tax)
6. Optionally qualify in your operating state

**Pros:**
- ✅ VCs' preferred structure
- ✅ Simple, well-understood
- ✅ No Canadian corporate overhead
- ✅ Stripe Atlas or Clerky can do this in days

**Cons:**
- ❌ US tax filing requirements (federal + state)
- ❌ You're non-resident, some complexity
- ❌ Potential double taxation without planning
- ❌ No Canadian R&D credits (SR&ED)
- ❌ Must maintain US presence for banking

**Best for**: Founders committed to US market, seeking significant VC

---

### Option 2: Canadian Federal Corporation (Canada Only)

**Structure:**
```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│                Federal Canadian Corporation                      │
│                (5D Labs Inc. - Canada)                          │
│                         │                                        │
│              ┌──────────┴──────────┐                            │
│              │                     │                            │
│         Founder(s)            Investors                         │
│         (Canada)              (Canada/Intl)                     │
│                                                                  │
│  • All IP in Canada                                             │
│  • Eligible for SR&ED credits                                   │
│  • Can still sell globally                                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Process:**
1. Incorporate federally via Corporations Canada (~$200)
2. Register in your province
3. Get business number from CRA
4. Open Canadian business bank account

**Pros:**
- ✅ Simple for Canadian founder
- ✅ **SR&ED tax credits** (up to 35% of R&D expenses refunded)
- ✅ Lower corporate tax rate (combined ~26.5% vs US ~21%)
- ✅ No US tax complexity
- ✅ Canadian banking is straightforward

**Cons:**
- ❌ Many US VCs won't invest directly
- ❌ Requires restructuring if you later want US VC
- ❌ US acquirers may discount valuation
- ❌ Stock option treatment differs

**Best for**: Bootstrapping, Canadian investors, smaller rounds (<$500K)

---

### Option 3: Hybrid Structure (US HoldCo + Canadian OpCo) 🔄 Complex but Flexible

**Structure:**
```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│                    Delaware C-Corp                               │
│                    (5D Labs, Inc.)                               │
│                    [Holding Company]                             │
│                         │                                        │
│              ┌──────────┴──────────┐                            │
│              │                     │                            │
│         Founder(s)            VC Investors                      │
│         (shares)              (shares)                          │
│                                                                  │
│                         │                                        │
│                         │ 100% owns                             │
│                         ▼                                        │
│              ┌─────────────────────┐                            │
│              │ Canadian Subsidiary │                            │
│              │ (5D Labs Canada ULC)│                            │
│              │ [Operating Company] │                            │
│              └─────────────────────┘                            │
│                         │                                        │
│              ┌──────────┴──────────┐                            │
│              │                     │                            │
│         Employees              Operations                       │
│         (Canada)               R&D, etc.                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**How it works:**
- US C-Corp is the "parent" that VCs invest in
- Canadian subsidiary does actual operations
- IP can be licensed between entities (transfer pricing)
- R&D done in Canada may qualify for SR&ED

**Pros:**
- ✅ VCs see Delaware C-Corp
- ✅ Can claim Canadian SR&ED credits
- ✅ Employees in Canada get Canadian benefits
- ✅ Potential tax optimization (carefully!)
- ✅ Flexibility for future expansion

**Cons:**
- ❌ Complex setup ($10-30K in legal fees)
- ❌ Ongoing compliance in both countries
- ❌ Transfer pricing documentation required
- ❌ Two sets of tax returns
- ❌ CRA scrutiny on intercompany arrangements

**Best for**: Significant R&D, want SR&ED credits, serious VC raise

---

### Option 4: Canadian Corp → US Flip Later

**Structure (Start):**
```
Start as Canadian → Later "flip" to US
         
Phase 1 (Now):              Phase 2 (Later):
┌────────────────┐          ┌────────────────┐
│ Canadian Corp  │    →     │ Delaware Corp  │
│ (bootstrapping)│          │ (VC round)     │
└────────────────┘          └────────────────┘
```

**How "Flipping" Works:**
1. Create new Delaware C-Corp
2. Exchange shares (Canadian shareholders get US shares)
3. Canadian corp becomes subsidiary (or winds up)
4. IP transfers to US parent

**Pros:**
- ✅ Start simple, complexify later
- ✅ Get SR&ED credits early (when most valuable)
- ✅ Defer US complexity until funded
- ✅ Validate business before restructuring

**Cons:**
- ❌ Flip costs $20-50K+ in legal/accounting
- ❌ Tax consequences on flip (potential capital gains)
- ❌ Timeline pressure when fundraising
- ❌ Due diligence on historical structure

**Best for**: Unsure about VC path, want to validate first

---

## Challenges for Canadian Founders

### 1. US Banking as Non-Resident

**The Problem:**
- US banks require US address, SSN, or significant US presence
- Some banks won't open accounts for non-residents
- Wire transfers can be complicated

**Solutions:**
- **Mercury**: Startup-friendly, works with non-residents
- **Brex**: Requires some US connection but flexible
- **Relay**: Another startup-focused option
- **Stripe Atlas**: Includes Mercury account setup
- Get US address via registered agent or virtual office

### 2. Tax Complexity

**US Obligations (as Delaware C-Corp):**
```
Federal:
- Form 1120 (corporate tax return) annually
- Estimated quarterly payments if profitable
- May have state filings too

Personal (as non-resident founder):
- Potential 1040-NR if you have US-source income
- W-8BEN for dividend withholding
- Canadian reporting of US company ownership
```

**Canadian Obligations:**
```
Personal:
- Report worldwide income on Canadian return
- Report foreign property (T1135) if >$100K CAD
- Potential double taxation (mitigated by treaty)
- Foreign affiliate reporting if >10% of foreign corp
```

**Key Point**: US-Canada tax treaty prevents most double taxation, 
but you need proper planning and reporting.

### 3. Stock Options & Equity

**US C-Corp:**
- ISOs (Incentive Stock Options): Not available to non-resident aliens
- NSOs (Non-Qualified Stock Options): Available, but different tax treatment
- RSUs: Complex tax timing for Canadians

**Canadian Employees of US Corp:**
- Options taxed as income in Canada when exercised
- Potential US withholding requirements
- Need proper tax equalization planning

### 4. Immigration Considerations

**Working for Your US Corp from Canada:**
- Generally fine - no US visa needed if you're in Canada
- Be careful about US "presence" for tax purposes
- Keep trips to US under 183 days/year

**If You Want to Move to US:**
- O-1 visa (extraordinary ability) - possible for founders
- L-1 visa (intracompany transfer) - requires 1 year at Canadian sub
- H-1B - lottery system, not ideal for founders
- EB-5 - investment-based green card ($800K+)

**For now**: Most Canadian founders stay in Canada, work remotely

---

## Comparison Matrix

| Factor | DE C-Corp | Canadian Corp | Hybrid | Flip Later |
|--------|-----------|---------------|--------|------------|
| **VC Friendliness** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Setup Cost** | $500-2K | $200-500 | $10-30K | $200 now, $30K+ later |
| **Ongoing Complexity** | Medium | Low | High | Low → High |
| **SR&ED Credits** | ❌ | ✅ | ✅ | ✅ early |
| **US Bank Access** | Yes | Difficult | Yes | Difficult → Yes |
| **Exit Simplicity** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Tax Optimization** | Limited | Good | Best | Good → Limited |

---

## Recommended Path

### If You're Confident About VC Path

**Go directly to Delaware C-Corp:**

1. **Week 1**: Use Stripe Atlas or Clerky (~$500)
   - Delaware incorporation
   - EIN assignment
   - Bank account setup (Mercury)
   - Basic legal docs

2. **Week 2-4**: Additional setup
   - Registered agent service
   - Virtual US address (for banking)
   - Proper founder equity documentation
   - 83(b) election if applicable

3. **Ongoing**: 
   - Quarterly bookkeeping
   - Annual tax returns
   - Maintain corporate formalities

**Cost**: ~$500 setup + ~$2-3K/year in compliance

### If You Want Flexibility

**Start Canadian, Plan for Flip:**

1. **Now**: Incorporate Canadian Federal Corp
   - Simple, cheap, get going
   - Claim SR&ED if doing R&D

2. **At Seed/Series A**: Evaluate flip
   - If raising $500K+ from US VCs, flip
   - Budget $30-50K for restructuring
   - Time it before term sheet signing

3. **Consider Hybrid** if:
   - Significant Canadian team staying long-term
   - R&D credits are substantial (>$100K/year)
   - You have budget for ongoing complexity

---

## Service Providers

### Incorporation Services

| Service | Cost | Best For |
|---------|------|----------|
| **Stripe Atlas** | $500 | Simple DE corp + banking |
| **Clerky** | $799+ | More legal doc customization |
| **Firstbase** | $399 | Budget option |
| **LegalZoom** | $299+ | Basic, less startup-focused |

### Cross-Border Specialists (Canada-US)

For hybrid structures or complex situations:

- **Osler, Hoskin & Harcourt** - Major Canadian firm, US practice
- **Blake, Cassels & Graydon** - Strong cross-border
- **Wilson Sonsini** - US firm with Canada practice
- **Gowling WLG** - Canadian with US presence

**Budget**: $10-30K for proper hybrid structure setup

### Accountants (Cross-Border)

- **BDO** - Both countries
- **Grant Thornton** - Both countries
- **MNP** - Strong Canadian startup practice
- Local CPA with cross-border experience

**Budget**: $3-10K/year for both country filings

---

## Frequently Asked Questions

### Can US VCs invest in Canadian corps?

**Technically yes, but:**
- Many funds have restrictions
- More due diligence required
- Often asked to flip before closing
- May get lower valuation

### Do I need to live in the US?

**No.** Many Canadian founders:
- Live in Canada
- Run US Delaware C-Corp
- Work remotely
- Visit US for meetings only

### What about Y Combinator?

**YC requires Delaware C-Corp** as a condition of investment. 
If YC is a goal, incorporate DE from the start.

### Can I convert Canadian corp to US later?

**Yes, but it's called a "flip"** and involves:
- Creating new US corp
- Share exchange (tax event!)
- IP transfer
- Legal fees ($20-50K)
- 2-3 months minimum

### What if I raise from Canadian VCs?

**Canadian corps work fine** for Canadian VCs like:
- BDC Venture Capital
- Real Ventures
- Georgian Partners
- Inovia Capital

These funds are comfortable with Canadian structures.

---

## Action Items

### Immediate (Before Fundraising)

- [ ] Decide on structure based on fundraising plans
- [ ] Consult with cross-border attorney (1-2 hour consult ~$500)
- [ ] Consult with cross-border accountant (understand tax implications)
- [ ] If DE C-Corp: Use Stripe Atlas or Clerky
- [ ] Set up proper banking

### Before Raising

- [ ] Ensure cap table is clean
- [ ] Proper founder agreements in place
- [ ] IP assignment to company (not personal)
- [ ] No prior investors with weird terms
- [ ] Corporate minutes up to date

### Ongoing

- [ ] Quarterly bookkeeping
- [ ] Annual tax returns (both countries if applicable)
- [ ] Maintain corporate formalities
- [ ] T1135 reporting if >$100K CAD in foreign property

---

## Resources

### Reading
- [Stripe Atlas Guide](https://stripe.com/atlas)
- [Clerky Formation](https://clerky.com)
- [YC Library - Should I Start an LLC or Corporation?](https://www.ycombinator.com/library)
- Canada-US Tax Treaty text

### Communities
- r/startups
- r/canadianstartups  
- Indie Hackers
- YC Startup School forums

---

## Potential Investors List

### US/Global - Developer Tools & Infrastructure VCs (Seed/Series A)

Based on 2024/2025 deal activity analysis:

| Investor | Stages | Focus | Check Size | Location | Contact |
|----------|--------|-------|------------|----------|---------|
| **Y Combinator** | Pre-Seed, Seed | DevTools, AI, Infra | $500K (standard) | SF Bay Area | [ycombinator.com/apply](https://www.ycombinator.com/apply) |
| **Pioneer Fund** | Pre-Seed, Seed | YC Alumni focused | $100K-500K | SF Bay Area | [pioneerfund.vc](https://www.pioneerfund.vc/) |
| **Sequoia Capital** | Seed → Growth | DevTools, AI, Enterprise | $1M-100M | SF Bay Area | [sequoiacap.com](https://www.sequoiacap.com/) |
| **Techstars** | Pre-Seed, Seed | DevTools, varies by program | $120K | Multiple (Toronto, others) | [techstars.com/accelerators](https://www.techstars.com/) |
| **Lightspeed Venture Partners** | Series A+ | DevTools, AI, Enterprise | $5M-50M | SF Bay Area | [lsvp.com](https://lsvp.com/) |
| **Antler** | Pre-Seed, Seed | DevTools, AI | $100K-500K | Multiple (incl. Canada) | [antler.co](https://www.antler.co/) |
| **Accel** | Series A+ | DevTools, SaaS, Enterprise | $10M-50M | SF Bay Area | [accel.com](https://www.accel.com/) |
| **Firestreak Ventures** | Pre-Seed, Seed | AI Infra, DevTools | $500K-3M | SF Bay Area | [firestreak.com](https://www.firestreak.com/) |
| **Index Ventures** | Seed → Growth | DevTools, Open Source | $1M-50M | SF / London | [indexventures.com](https://www.indexventures.com/) |
| **Felicis** | Seed → Series B | DevTools, Infrastructure | $1M-15M | SF Bay Area | [felicis.com](https://www.felicis.com/) |
| **SV Angel** | Pre-Seed, Seed | DevTools, AI | $100K-1M | SF Bay Area | [svangel.com](https://svangel.com/) |
| **Andreessen Horowitz (a16z)** | Seed → Growth | AI, Infra, Enterprise | $5M-100M | SF Bay Area | [a16z.com](https://a16z.com/) |
| **Matrix Partners** | Seed, Series A | DevTools, AI, Enterprise | $100K-10M | SF / Boston | [matrix.vc](https://www.matrix.vc/) |
| **Menlo Ventures** | Seed → Series B | DevTools, AI, SaaS | $8M-15M | SF Bay Area | [menlovc.com](https://www.menlovc.com/) |
| **Insight Partners** | Series A, B | DevTools, Enterprise | $10M-350M | New York | [insightpartners.com](https://www.insightpartners.com/) |
| **Engineering Capital** | Seed | Infra, Deep Tech | $1M-3M | SF Bay Area | Seed fund for technical founders |
| **Eight Capital** | Seed | YC Companies only | $500K-2M | SF Bay Area | [eightcapital.com](https://www.eightcapital.com/) |

**Notes:**
- Firestreak: Known for fast decisions (2-3 weeks to term sheet)
- Felicis: Also fast, has "Founder Development Pledge"
- Pioneer Fund: 450+ YC alumni investing together

### Canadian Venture Capital Firms

| Investor | Stages | Focus | Check Size | Location | Contact |
|----------|--------|-------|------------|----------|---------|
| **BDC Ventures** | Seed → Growth | Clean Tech, Deep Tech, Women | $500K-10M | Montreal | [bdc.ca/venture](https://www.bdc.ca/en/bdc-capital/venture-capital) |
| **Golden Ventures** | Pre-Seed, Seed | Industry agnostic | $500K-2M | Toronto | [golden.ventures](https://golden.ventures/) |
| **Inovia Capital** | Seed → Growth | B2B SaaS, Marketplaces | $1M-50M | Montreal | [inovia.vc](https://www.inovia.vc/) |
| **Real Ventures** | Pre-Seed, Seed, A | Tech, Bold ideas | $500K-5M | Montreal | [realventures.com](https://realventures.com/) |
| **Georgian** | Series A, B | B2B SaaS, AI | $25M-75M | Toronto | [georgian.io](https://georgian.io/) |
| **Panache Ventures** | Pre-Seed, Seed | Industry agnostic | $100K-1M | Montreal | [panache.vc](https://www.panache.vc/) |
| **ArcTern Ventures** | Seed → Series B | Climate Tech | $1M-10M | Toronto / SF | [arcternventures.com](https://www.arcternventures.com/) |
| **Relay Ventures** | Pre-Seed, Seed | Industry agnostic | $500K-3M | Toronto | [relay.vc](https://relay.vc/) |
| **Alate Partners** | Seed, Series A | Real Estate Tech | $1M-5M | Toronto | [alatepartners.com](https://alatepartners.com/) |
| **BlueSky Equities** | Seed, Series A | B2B SaaS | $250K-2M | Calgary | [blueskyequities.com](https://www.blueskyequities.com/) |

**Canadian Accelerators:**
- **Creative Destruction Lab (CDL)** - Toronto, Vancouver, Montreal - Science-based startups
- **FounderFuel** - Montreal - Demo Day with top Canadian VCs
- **DMZ (Toronto Metropolitan University)** - Top Canadian tech incubator
- **MaRS Discovery District** - Toronto - Health, cleantech, fintech, enterprise
- **District 3** - Montreal - Tech innovation

**Government Programs:**
- **SR&ED** - Up to 35% tax credit on R&D expenses
- **VCAP** - Government co-investment with private VCs
- **Innovative Solutions Canada** - Contracts for innovative solutions

### Angel Investors - Developer Tools Focus

| Investor | Stages | Focus | Check Size | Contact |
|----------|--------|-------|------------|---------|
| **Amjad Masad** | Pre-Seed, Seed | DevTools, AI | $50K-500K | [@amasad](https://twitter.com/amasad) - Replit founder |
| **Ilya Sukhar** | Seed, Series A | DevTools, AI, Infra | $100K-10M | [LinkedIn](https://www.linkedin.com/in/ilyasukhar/) - Parse founder, Matrix |
| **Alfred Lin** | Seed, Series A | DevTools, Enterprise | $1M-10M | [LinkedIn](https://www.linkedin.com/in/linalfred/) - Sequoia, ex-Zappos COO |
| **Matt Murphy** | Seed → Series B | DevTools, AI, Infra | $8M-15M | [LinkedIn](https://www.linkedin.com/in/matt-murphy-0415543/) - Menlo Ventures |
| **John O'Farrell** | Seed, Series A | DevTools, Enterprise | $1M-100M | [LinkedIn](https://www.linkedin.com/in/john-o-farrell-b7252/) - a16z |
| **Dan Scholnick** | Seed, Series A | DevTools, Cloud | $100K-5M | [LinkedIn](https://www.linkedin.com/in/dscholnick/) - Four Rivers Group |
| **Marvin Liao** | Pre-Seed, Seed | DevTools, SaaS | $10K-500K | [LinkedIn](https://www.linkedin.com/pub/marvin-liao/0/5/630/) - ex-500 Startups |
| **Stan Reiss** | Pre-Seed → B | DevTools, AI, Infra | $100K-10M | [LinkedIn](https://www.linkedin.com/in/stanreiss/) - Matrix Partners |
| **Creighton Hicks** | Seed, Series A | DevTools, Cloud, Security | $250K-4M | [LinkedIn](https://www.linkedin.com/in/creightonhicks/) - LiveOak Ventures |
| **Rama Sekhar** | Seed → Series B | DevTools, AI, Security | $1M-30M | [LinkedIn](https://www.linkedin.com/in/ramasekhar/) - Norwest |

### Outreach Strategy

**Tier 1: Direct Relevance (Infrastructure/DevTools focus)**
1. Y Combinator - Apply to batch
2. Firestreak Ventures - Fast decisions, backed Anthropic
3. Pioneer Fund - If you have YC network connections
4. Engineering Capital - Seed fund for infra
5. BDC Ventures (if staying Canadian) - Deep Tech fund

**Tier 2: Developer Tools Generalists**
1. Felicis - Fast decisions, founder-friendly
2. Matrix Partners (via Stan Reiss or Ilya Sukhar)
3. Inovia Capital - Canadian but invests globally
4. Golden Ventures - Canadian seed leader

**Tier 3: Larger Checks (Series A ready)**
1. Sequoia Capital
2. a16z
3. Accel
4. Index Ventures

### Warm Introduction Paths

**Via Open Source Community:**
- Contribute to popular projects, build relationships
- Speak at conferences (KubeCon, DevOpsDays)
- Engage with VC-backed founders in same space

**Via Accelerators:**
- YC → Access to Pioneer Fund, Eight Capital, SV Angel
- Techstars → Network of mentors and VCs
- CDL → Canadian investor network

**Via Twitter/X:**
Many devtools investors are active on X:
- [@amasad](https://twitter.com/amasad) - Amjad Masad
- [@ilyasukhar](https://twitter.com/ilyasukhar) - Ilya Sukhar
- [@paborenstein](https://twitter.com/paborenstein) - Peter Aborenstein (Index)
- Build in public, share technical content, engage genuinely

### What's Getting Funded (2024/2025)

Based on deal analysis:
- **48% AI-related** - AI infrastructure, dev productivity with AI
- **39% Security** - DevSecOps, identity management
- **30% Non-AI/Security** - Traditional infra, databases, dev tools

**Hot themes:**
- AI-powered development tools
- Cloud infrastructure optimization
- Developer experience/productivity
- Self-hosted/on-prem alternatives (your positioning!)

---

*Last updated: November 2024*

*This document is informational only. Consult qualified legal and tax 
professionals for advice specific to your situation.*

