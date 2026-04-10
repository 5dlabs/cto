# SWARM Puzzle #2 Final Verification

## Summary
- Base URL: `https://swarm.thecanteenapp.com/`
- Key: `7GysZCDoTdz1ECULFnqMpZ3CDnHMziN978u6mXAUYkQk`
- Best-supported phrase: `Real-Time Agent Coordination`
- Scope stayed on `swarm.thecanteenapp.com` artifacts and SWARM-local git/text evidence only.

## Live Discovery
- `live_homepage` -> status `200` `text/html` from `https://swarm.thecanteenapp.com/`
- `live-git-fetch-head` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/FETCH_HEAD`
- `live-git-head` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/HEAD`
- `live-git-config` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/config`
- `live-git-description` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/description`
- `live-git-logs-head` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/logs/HEAD`
- `live-git-logs-refs-heads-main` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/logs/refs/heads/main`
- `live-git-packed-refs` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/packed-refs`
- `live-git-refs-heads-main` -> status `200` `application/octet-stream` from `https://swarm.thecanteenapp.com/.git/refs/heads/main`
- `live-homepage-referrer-luma` -> status `200` `text/html` from `https://swarm.thecanteenapp.com/?referrer=luma`
- `live-canteen-logo-png` -> status `200` `image/png` from `https://swarm.thecanteenapp.com/canteen-logo.png`
- `probe_robots-txt` -> status `404` `text/html` from `https://swarm.thecanteenapp.com/robots.txt`
- `probe_sitemap-xml` -> status `404` `text/html` from `https://swarm.thecanteenapp.com/sitemap.xml`
- `probe_stats-html` -> status `404` `text/html` from `https://swarm.thecanteenapp.com/stats.html`
- `live-swarm-v1-html` -> status `200` `text/html` from `https://swarm.thecanteenapp.com/swarm-v1.html`

## Hidden Git Commit
- Hidden HEAD commit: `678f46bb4e72d7fb0b4df919064ff38dc8b80a5b`
- Hidden commit message: `RFB blurb`
- Parent commit: `7ce43ed440f2b45eb4f2f62d38707e66a9ad5bb6`
- Added exact 3-word phrases in hidden commit: `0`
- Removed exact 3-word phrases in hidden commit: `0`

### Diff Excerpt
```diff
--- parent

+++ hidden

@@ -588,6 +588,7 @@

   </div>
   <div class="nav-right">
     <div class="nav-links">
+      <a href="#next-steps" class="hide-mobile">Next Steps</a>
       <a href="#prizes" class="hide-mobile">Prizes</a>
       <a href="#ideas">Ideas</a>
       <a href="#examples" class="hide-mobile">Research</a>
@@ -743,7 +744,7 @@

     <div class="sec-label-num">03</div>
   </div>
   <div class="sec-body">
-    <p class="rfb-intro">RFBs — "Requests for Builders" — are our version of <a href="https://www.ycombinator.com/rfs" target="_blank">YC's Requests for Startups</a>. Five open problems in the agent economy that we think are worth solving. If one excites you, treat it as extra validation to dive in — but you don't need to work on any of these to participate in SWARM. <strong>The best submissions always start from a specific frustration.</strong></p>
+    <p class="rfb-intro">RFBs — "Requests for Builders" — are our version of <a href="https://www.ycombinator.com/rfs" target="_blank">YC's Requests for Startups</a>. Five open problems in the agent economy that we think are worth solving. If one excites you, treat it as extra validation to dive in — but you don't need to work on any of these to participate in SWARM. <strong>The best submissions always are what you care most about.</strong></p>
     <div class="rfb-list">
       <div class="rfb-item" id="rfb-1">
         <div class="rfb-trigger">
```

## Candidate Ranking
| Phrase | Homepage score | Weighted score | Coverage | Hidden-only | Partner-like | Sources |
| --- | ---: | ---: | ---: | --- | --- | --- |
| `Real-Time Agent Coordination` | 12 | 92 | 8 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_mobile_fixed, repo_swarm_v1` |
| `Multi-Agent Landscape 2026` | 6 | 123 | 8 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_mobile_fixed, repo_swarm_v1` |
| `YOUR PRIORITY PASS` | 6 | 34 | 1 | `False` | `False` | `homepage_visible` |
| `Agent Attention Markets` | 4 | 63 | 8 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_mobile_fixed, repo_swarm_v1` |
| `Join our Discord` | 4 | 36 | 4 | `False` | `False` | `homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage` |
| `Real-time network data` | 4 | 35 | 8 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_mobile_fixed, repo_swarm_v1` |
| `Emergent Agent Economies` | 2 | 43 | 8 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_mobile_fixed, repo_swarm_v1` |
| `The Canteen network` | 2 | 41 | 8 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_mobile_fixed, repo_swarm_v1` |
| `Online 4 weeks` | 2 | 29 | 8 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_mobile_fixed, repo_swarm_v1` |
| `Apply to join` | 2 | 29 | 7 | `False` | `False` | `git_hidden_swarm_v1, git_parent_swarm_v1, homepage_visible, live-homepage-referrer-luma, live-swarm-v1-html, live_homepage, repo_swarm_v1` |

## Clue Hits
- `repo_swarm_v1` contains `unlock` `1` time(s)
- `repo_swarm_v1` contains `passphrase` `2` time(s)
- `repo_swarm_v1` contains `special` `3` time(s)
- `repo_mobile_fixed` contains `unlock` `1` time(s)
- `repo_mobile_fixed` contains `passphrase` `2` time(s)
- `repo_mobile_fixed` contains `special` `3` time(s)
- `live_homepage` contains `unlock` `1` time(s)
- `live_homepage` contains `passphrase` `2` time(s)
- `live_homepage` contains `special` `3` time(s)
- `live-homepage-referrer-luma` contains `unlock` `1` time(s)
- `live-homepage-referrer-luma` contains `passphrase` `2` time(s)
- `live-homepage-referrer-luma` contains `special` `3` time(s)
- `live-swarm-v1-html` contains `unlock` `1` time(s)
- `live-swarm-v1-html` contains `passphrase` `2` time(s)
- `live-swarm-v1-html` contains `special` `3` time(s)
- `git_hidden_swarm_v1` contains `unlock` `1` time(s)
- `git_hidden_swarm_v1` contains `passphrase` `2` time(s)
- `git_hidden_swarm_v1` contains `special` `3` time(s)
- `git_parent_swarm_v1` contains `unlock` `1` time(s)
- `git_parent_swarm_v1` contains `passphrase` `2` time(s)
- `git_parent_swarm_v1` contains `special` `3` time(s)

## Conclusion
- `Real-Time Agent Coordination` remains the top candidate because it wins the homepage-visible set, survives the SWARM-only weighting, and no hidden git artifact introduces a stronger 3-word phrase.
